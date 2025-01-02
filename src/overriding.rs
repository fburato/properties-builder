use crate::model::Property;
use std::cmp::Ordering;
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
        let prefix_match = prefix.as_ref().to_string();
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

        const PREFIX: &str = "PREFIX_";
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
    environment: Environment,
}

impl CustomCaseSensitiveStyleOverrider {
    pub fn new<S: AsRef<str>>(
        character_replacement_map: HashMap<char, S>,
        environment: Environment,
    ) -> CustomCaseSensitiveStyleOverrider {
        CustomCaseSensitiveStyleOverrider {
            character_replacement_map: character_replacement_map
                .iter()
                .map(|(key, value)| (key.clone(), value.as_ref().to_string()))
                .collect(),
            environment,
        }
    }

    fn process_character(&self, c: char) -> String {
        match self.character_replacement_map.get(&c) {
            None => {
                let mut s = String::new();
                s.push(c);
                s
            }
            Some(s) => s.clone(),
        }
    }
}
impl Overrider for CustomCaseSensitiveStyleOverrider {
    fn resolve_substitution<S: AsRef<str>>(&self, key: S, prefix: Option<S>) -> Option<&str> {
        let mut transformed_key: String = prefix
            .map(|s| s.as_ref().to_string())
            .unwrap_or("".to_string());
        for c in key.as_ref().chars() {
            transformed_key = transformed_key + self.process_character(c).as_str()
        }
        self.environment.get(transformed_key)
    }

    fn generate_additions<S: AsRef<str>>(&self, prefix: S) -> Vec<Property> {
        let prefix_match = prefix.as_ref().to_string();
        let prefixed_entries: HashMap<&str, &str> = self
            .environment
            .env
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix_match))
            .map(|(key, value)| (key.as_ref(), value.as_ref()))
            .collect();
        let mut properties: Vec<Property> = Vec::new();
        let reverse_replacement_index: HashMap<String, char> = self
            .character_replacement_map
            .iter()
            .map(|(key, value)| (value.clone(), key.clone()))
            .collect();
        let mut replacement_descending: Vec<&str> = reverse_replacement_index
            .keys()
            .map(|s| s.as_str())
            .collect();
        replacement_descending.sort_by(|a, b| {
            if a.len() < b.len() {
                Ordering::Greater
            } else if a.len() == b.len() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        });
        for (key, value) in prefixed_entries {
            let prefixless_key = key.trim_start_matches(&prefix_match);
            let mut start: usize = 0;
            let mut replaced_key = "".to_string();
            while start < prefixless_key.len() {
                let mut found_match: Option<&str> = None;
                for candidate in &replacement_descending {
                    if start + candidate.len() <= prefixless_key.len()
                        && **candidate == prefixless_key[start..start + candidate.len()]
                    {
                        found_match = Some(*candidate);
                        break;
                    }
                }
                if let Some(existing_key) = found_match {
                    replaced_key.push(reverse_replacement_index.get(existing_key).unwrap().clone());
                    start = start + existing_key.len();
                } else {
                    replaced_key.push_str(prefixless_key[start..start + 1].to_string().as_str());
                    start = start + 1;
                }
            }
            properties.push(Property::new(replaced_key.as_str(), value))
        }
        properties
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
        let environment = Environment::new(&hashmap! {"foo" => "bar"});

        let testee =
            CustomCaseSensitiveStyleOverrider::new(replacement.clone(), environment.clone());

        assert_eq!(replacement, testee.character_replacement_map);
        assert_eq!(environment.env, testee.environment.env);
    }

    fn make(environment: HashMap<&str, &str>) -> CustomCaseSensitiveStyleOverrider {
        let replacement: HashMap<char, String> = hashmap! {
            '.' => "_".to_string(),
            '-' => "__".to_string(),
            '_' => "___".to_string()
        };
        CustomCaseSensitiveStyleOverrider::new(replacement, Environment::new(&environment))
    }

    #[cfg(test)]
    mod resolve_tests {
        use super::*;
        #[test]
        fn should_replace_single_word() {
            let testee = make(hashmap! {
                "foo" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_return_none_for_non_existing_replacement() {
            let testee = make(hashmap! {
                "foo" => "value for foo"
            });

            assert_eq!(testee.resolve_substitution("bar", None), None);
        }

        #[test]
        fn should_use_case_sensitive_resolution_for_key() {
            let testee = make(hashmap! {
                "fOo" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("fOo", None),
                Some("value for foo")
            );
            assert_eq!(testee.resolve_substitution("foo", None), None);
        }

        #[test]
        fn should_use_all_character_replacements_for_lookup() {
            let testee = make(hashmap! {
                "foo_bar__baz___foobarbaz" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar-baz_foobarbaz", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_replace_multiple_occurrences_of_the_same_charater() {
            let testee = make(hashmap! {
                "foo_bar_baz" => "value for foo"
            });

            assert_eq!(
                testee.resolve_substitution("foo.bar.baz", None),
                Some("value for foo")
            );
        }

        #[test]
        fn should_search_for_variable_with_prefix_if_specified() {
            let testee = make(hashmap! {
                "foo_bar" => "value1",
                "PREFIX_foo_bar" => "value2",
                "foo" => "value3",
                "PREFIX_foo" => "value4"
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

        const PREFIX: &str = "PREFIX_";
        #[test]
        fn should_convert_simple_prefixed_keys_with_values() {
            let testee = make(hashmap! {
                "PREFIX_foo" => "value1",
                "PREFIX_bar" => "value2"
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
                "PREFIX_foo" => "value3",
                "NOT_PREFIX_bar" => "value2"
            });

            assert_contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![Property::new("foo", "value3")],
            );
        }

        #[test]
        fn should_convert_all_converted_characters_into_original_characters() {
            let testee = make(hashmap! {
                "PREFIX_foo_bar__baz___barfoo" => "value5",
                "PREFIX_bar____foo_baz__foobar" => "value1"
            });

            assert_contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![
                    Property::new("foo.bar-baz_barfoo", "value5"),
                    Property::new("bar_.foo.baz-foobar", "value1"),
                ],
            );
        }

        #[test]
        fn should_convert_handle_repeated_occurrences_conversion() {
            let testee = make(hashmap! {
                "PREFIX_foo___bar__baz___barfoo_" => "value5",
                "PREFIX_bar__foo__baz__foobar__" => "value1"
            });

            assert_contains_exactly_in_any_order(
                testee.generate_additions(PREFIX),
                vec![
                    Property::new("foo_bar-baz_barfoo.", "value5"),
                    Property::new("bar-foo-baz-foobar-", "value1"),
                ],
            );
        }
    }
}
