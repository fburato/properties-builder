use crate::model::Property;
use std::collections::HashMap;

pub trait Overrider {
    fn resolve_substitution<S: AsRef<str>>(&self, key: S, prefix: Option<S>) -> Option<&str>;
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
    fn resolve_substitution<S: AsRef<str>>(&self, key: S, prefix: Option<S>) -> Option<&str> {
        let variable_to_resolve = prefix
            .map(|s| s.as_ref().to_string())
            .unwrap_or("".to_string())
            + key
                .as_ref()
                .replace(".", "_")
                .replace("-", "_")
                .to_uppercase()
                .as_str();
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

            assert_eq!(
                testee.resolve_substitution("foo", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_return_none_for_non_existing_replacement() {
            let testee = make(hashmap! {
                "FOO" => "value for foo"
            });

            assert_eq!(testee.resolve_substitution("bar", None), None);
        }

        #[test]
        fn should_ignore_capitalisation_of_key() {
            let testee = make(hashmap! {
                "FOO" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("fOo", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_dots_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_multiple_dots_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar.baz", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_dashes_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo-bar", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_multiple_dashes_with_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo-bar-baz", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_not_replace_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo_bar", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_not_replace_multiple_underscores_for_lookup() {
            let testee = make(hashmap! {
                "FOO_BAR_BAZ" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo_bar_baz", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_search_for_variable_with_prefix_if_specified() {
            let testee = make(hashmap! {
                "FOO_BAR" => "value1",
                "PREFIX_FOO_BAR" => "value2",
                "FOO" => "value3",
                "PREFIX_FOO" => "value4"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar", Some("PREFIX_")),
                Some("value2")
            );
            assert_eq!(
                testee.resolve_substitution("foo", Some("PREFIX_")),
                Some("value4")
            );
        }
    }

    #[cfg(test)]
    mod addition_tests {
        use super::*;
        use crate::test_utils::assert_contains_exactly_in_any_order;
        use std::fmt::Debug;

        const PREFIX: &str = "PREFIX";
        #[test]
        fn should_convert_simple_prefixed_keys_with_values_to_lowercase_key() {
            let testee = make(hashmap! {
                "PREFIX_FOO" => "value1",
                "PREFIX_BAR" => "value2"
            });

            assert_contains_exactly_in_any_order(
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

            assert_contains_exactly_in_any_order(
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

            assert_contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![
                    Property::new("foo.baz", "value5"),
                    Property::new("bar.foo", "value1"),
                ],
            );
        }
    }
}

pub struct CustomCaseSensitiveStyleOverrider {
    character_replacement_map: HashMap<char, String>,
}

impl CustomCaseSensitiveStyleOverrider {
    pub fn new<S: AsRef<str>>(
        character_replacement_map: HashMap<char, S>,
    ) -> CustomCaseSensitiveStyleOverrider {
        todo!()
    }
}

#[cfg(test)]
mod custom_case_sensitive_style_overrider {
    use super::*;
    #[test]
    fn new_should_create_expected_type() {
        let replacement: HashMap<char, String> = hashmap! {
            '.' => "_".to_string(),
            '-' => "__".to_string(),
            '_' => "___".to_string()
        };

        let testee = CustomCaseSensitiveStyleOverrider::new(replacement.clone());

        assert_eq!(replacement, testee.character_replacement_map);
    }

    #[cfg(test)]
    mod resolve_tests {}
}
