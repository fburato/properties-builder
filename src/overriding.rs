use crate::model::Property;
use std::collections::HashMap;

pub trait Overrider {
    fn resolve_substitution<S: AsRef<str>>(&self, key: S) -> Option<&str>;
    fn generate_additions<S: AsRef<str>>(&self, prefix: S) -> Vec<Property>;
}

#[derive(Clone)]
pub struct Environment {
    env: HashMap<String, String>,
}

impl Environment {
    pub fn new<S: AsRef<str>>(map: &HashMap<S, S>) -> Environment {
        Environment {
            env: map
                .iter()
                .map(|(key, value)| (key.as_ref().to_string(), value.as_ref().to_string()))
                .collect(),
        }
    }

    fn get<S: AsRef<str>>(&self, variable: S) -> Option<&str> {
        self.env.get(variable.as_ref()).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod environment_tests {
    use crate::overriding::Environment;
    use std::collections::HashMap;

    #[test]
    fn new_should_construct_expected_environment() {
        let map: HashMap<String, String> = hashmap! {
          "foo".to_string() => "value".to_string()
        };
        let environment = Environment::new(&map);

        assert_eq!(environment.env, map.clone());
    }

    #[test]
    fn new_should_construct_environment_from_string_refs() {
        let testee = Environment::new(&hashmap! {
            "foo" => "value"
        });

        assert_eq!(
            testee.env,
            hashmap! {
              "foo".to_string() => "value".to_string()
            }
        );
    }

    #[test]
    fn get_should_return_none_if_variable_not_defined() {
        let testee = Environment::new(&hashmap! {
            "foo" => "bar"
        });

        assert_eq!(testee.get("baz"), None);
    }

    #[test]
    fn get_should_return_some_if_variable_defined() {
        let testee = Environment::new(&hashmap! {
            "foo" => "bar"
        });

        assert_eq!(testee.get("foo"), Some("bar"));
    }
}

pub struct SpringStyleOverrider {
    env: Environment,
}

impl SpringStyleOverrider {
    fn new(env: Environment) -> SpringStyleOverrider {
        SpringStyleOverrider { env }
    }
}

impl Overrider for SpringStyleOverrider {
    fn resolve_substitution<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let variable_to_resolve = key
            .as_ref()
            .replace(".", "_")
            .replace("-", "_")
            .to_uppercase();
        self.env.get(variable_to_resolve)
    }

    fn generate_additions<S: AsRef<str>>(&self, prefix: S) -> Vec<Property> {
        let prefix_match = prefix.as_ref().to_string() + "_";
        let prefixed_entries: HashMap<&str, &str> = self
            .env
            .env
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix_match))
            .map(|(key, value)| (key.as_ref(), value.as_ref()))
            .collect();
        prefixed_entries
            .into_iter()
            .map(|(key, value)| {
                let new_key = key
                    .trim_start_matches(&prefix_match)
                    .replace("_", ".")
                    .to_lowercase();
                Property::new(new_key.as_str(), value)
            })
            .collect()
    }
}

#[cfg(test)]
mod spring_style_overrider_tests {
    use super::*;
    #[test]
    fn new_should_generate_expected_instance() {
        let map = hashmap! {
            "foo".to_string() => "bar".to_string()
        };
        let environment = Environment::new(&map);
        let instance = SpringStyleOverrider::new(environment);

        assert_eq!(instance.env.env, map);
    }

    fn make(map: HashMap<&str, &str>) -> SpringStyleOverrider {
        SpringStyleOverrider::new(Environment::new(&map))
    }

    #[cfg(test)]
    mod resolve_tests {
        use super::*;
        #[test]
        fn should_replace_single_word_with_capital_letters() {
            let testee = make(hashmap! {
                "FOO" => "value for foo"
            });

            assert_eq!(testee.resolve_substitution("foo"), Some("value for foo"));
        }

        #[test]
        fn should_return_none_for_non_existing_replacement() {
            let testee = make(hashmap! {
                "FOO" => "value for foo"
            });

            assert_eq!(testee.resolve_substitution("bar"), None);
        }

        #[test]
        fn should_ignore_capitalisation_of_key() {
            let testee = make(hashmap! {
                "FOO" => "value for foo"
            });

            assert_eq!(testee.resolve_substitution("fOo"), Some("value for foo"));
        }

        #[test]
        fn should_replace_dots_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar"),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_multiple_dots_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar.baz"),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_dashes_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo-bar"),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_multiple_dashes_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo-bar-baz"),
                Some("value for foo")
            );
        }

        #[test]
        fn should_not_replace_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo_bar"),
                Some("value for foo")
            );
        }

        #[test]
        fn should_not_replace_multiple_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo_bar_baz"),
                Some("value for foo")
            );
        }
    }

    #[cfg(test)]
    mod addition_tests {
        use super::*;
        use std::collections::HashSet;
        use std::fmt::Debug;

        const PREFIX: &str = "PREFIX";

        fn contains_exactly_in_any_order<T: PartialEq + Debug>(actual: Vec<T>, expected: Vec<T>) {
            let mut matched_indexes_in_actual: HashSet<usize> = HashSet::new();
            let mut unmatched_indexes_in_expected: HashSet<usize> = HashSet::new();

            for (i, e) in expected.iter().enumerate() {
                let mut actual_index: Option<usize> = None;
                for (j, a) in actual.iter().enumerate() {
                    if matched_indexes_in_actual.contains(&j) {
                        continue;
                    }
                    if e == a {
                        matched_indexes_in_actual.replace(j);
                        actual_index = Some(j);
                        break;
                    }
                }
                if actual_index.is_none() {
                    unmatched_indexes_in_expected.replace(i);
                }
            }
            let mut excess: Vec<usize> = Vec::new();
            for (i, _) in actual.iter().enumerate() {
                if !matched_indexes_in_actual.contains(&i) {
                    excess.push(i);
                }
            }
            let mut result: String = "".to_string();
            let comma_separator = ", ";
            if !unmatched_indexes_in_expected.is_empty() {
                result = result + "The following elements where expected but not found:\n";
                let mut separator = "[";
                for i in unmatched_indexes_in_expected.iter() {
                    result = result + separator + format!("{:?}", expected[*i]).as_str();
                    separator = comma_separator;
                }
                result = result + "]\n"
            }
            if !excess.is_empty() {
                result = result + "The following elements where not expected:\n";
                let mut separator = "[";
                for a in excess {
                    result = result + separator + format!("{:?}", actual[a]).as_str();
                    separator = comma_separator;
                }
                result = result + "]";
            }
            if result != "" {
                panic!("{}", result);
            }
        }
        #[test]
        fn should_convert_simple_prefixed_keys_with_values_to_lowercase_key() {
            let testee = make(hashmap! {
                "PREFIX_FOO" => "value1",
                "PREFIX_BAR" => "value2"
            });

            contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![
                    Property::new("foo", "value1"),
                    Property::new("bar", "value2"),
                ],
            );
        }

        #[test]
        fn should_ignore_non_prefixed_key() {
            let testee = make(hashmap! {
                "PREFIX_FOO" => "value3",
                "NOT_PREFIX_BAR" => "value2"
            });

            contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![Property::new("foo", "value3")],
            );
        }

        #[test]
        fn should_convert_underscores_in_dots_for_keys() {
            let testee = make(hashmap! {
                "PREFIX_FOO_BAZ" => "value5",
                "PREFIX_BAR_FOO" => "value1"
            });

            contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![
                    Property::new("foo.baz", "value5"),
                    Property::new("bar.foo", "value1"),
                ],
            );
        }
    }
}
