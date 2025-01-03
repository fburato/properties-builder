use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: String,
}

impl Property {
    pub fn new<S: AsRef<str>>(key: S, value: S) -> Property {
        Property {
            key: key.as_ref().to_string(),
            value: value.as_ref().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum InternalError {
    ParseError { line_num: i32, message: String },
    ArgumentValidationErrors(Vec<String>),
    FileAccessError(io::Error),
}

impl InternalError {
    pub fn parse_error<S: AsRef<str>>(line_num: i32, message: S) -> InternalError {
        InternalError::ParseError {
            line_num,
            message: message.as_ref().to_string(),
        }
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalError::ParseError { line_num, message } => f.write_str(
                format!("cannot parse property at line {}: {}", line_num, message).as_str(),
            ),
            InternalError::ArgumentValidationErrors(messages) => f.write_str(
                format!(
                    "invalid arguments:\n{}",
                    messages
                        .iter()
                        .map(|s| "- ".to_string() + s)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
                .as_str(),
            ),
            InternalError::FileAccessError(io_error) => {
                f.write_str(format!("file access error: {}", io_error).as_str())
            }
        }
    }
}

impl From<io::Error> for InternalError {
    fn from(value: io::Error) -> Self {
        InternalError::FileAccessError(value)
    }
}

impl Error for InternalError {}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn new_should_construct_the_expected_property() {
        let property = Property::new("foo", "bar");

        assert_eq!(property.key, "foo");
        assert_eq!(property.value, "bar");
    }

    #[test]
    fn equals_should_be_well_implemented() {
        let p1 = Property::new("1", "2");
        let p2 = Property::new("1", "2");
        let p3 = Property::new("1", "3");
        let p4 = Property::new("4", "2");

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
        assert_ne!(p1, p4);

        // commutative
        assert_eq!(p2, p1);
        assert_ne!(p3, p1);
        assert_ne!(p4, p1);
    }
}

#[cfg(test)]
mod error_tests {
    #[cfg(test)]
    mod parse_error_tests {
        use crate::model::InternalError;
        use crate::model::InternalError::ParseError;

        #[test]
        fn parse_error_should_product_the_expected_error() {
            let parse_error = InternalError::parse_error(42, "foobar");

            match parse_error {
                ParseError { line_num, message } => {
                    assert_eq!(line_num, 42);
                    assert_eq!(message, "foobar");
                }
                _ => assert!(false),
            }
        }
        #[test]
        fn fmt_should_produce_the_expected_error_for_parse_error() {
            let error = InternalError::parse_error(45, "message");

            let result = format!("{}", &error);

            assert_eq!(result, "cannot parse property at line 45: message");
        }

        #[test]
        fn fmt_should_produce_the_expected_error_for_argument_validation_error() {
            let error =
                InternalError::ArgumentValidationErrors(vec!["one".to_string(), "two".to_string()]);

            let result = format!("{}", &error);

            assert_eq!(result, "invalid arguments:\n- one\n- two")
        }

        fn assert_parse_error_equal(actual: &InternalError, expected: &InternalError) {
            match actual {
                ParseError { line_num, message } => {
                    let (actual_line_num, actual_message) = (line_num, message);
                    match expected {
                        ParseError { line_num, message } => {
                            let (expected_line_num, expected_message) = (line_num, message);
                            assert_eq!(actual_line_num, expected_line_num);
                            assert_eq!(actual_message, expected_message);
                        }
                        _ => panic!("expected value is not ParseError"),
                    }
                }
                _ => panic!("actual is not ParseError"),
            }
        }

        fn assert_parse_error_not_equal(actual: &InternalError, expected: &InternalError) {
            match actual {
                ParseError { line_num, message } => {
                    let (actual_line_num, actual_message) = (line_num, message);
                    match expected {
                        ParseError { line_num, message } => {
                            let (expected_line_num, expected_message) = (line_num, message);
                            assert_ne!(
                                (actual_line_num, actual_message),
                                (expected_line_num, expected_message)
                            );
                        }
                        _ => panic!("expected value is not ParseError"),
                    }
                }
                _ => panic!("actual is not ParseError"),
            }
        }

        #[test]
        fn eq_should_be_well_implemented_for_parse_error() {
            let e1 = InternalError::parse_error(23, "foo");
            let e2 = InternalError::parse_error(23, "foo");
            let e3 = InternalError::parse_error(55, "foo");
            let e4 = InternalError::parse_error(23, "bar");

            assert_parse_error_equal(&e1, &e2);
            assert_parse_error_not_equal(&e1, &e3);
            assert_parse_error_not_equal(&e1, &e4);

            assert_parse_error_equal(&e2, &e1);
            assert_parse_error_not_equal(&e3, &e1);
            assert_parse_error_not_equal(&e4, &e1);
        }
    }
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long)]
    pub output_file: Option<String>,
    #[arg(long, short)]
    pub spring: bool,
    #[arg(long, short)]
    pub prefix: String,
    #[arg(long, short)]
    pub replacement: Vec<String>,
    pub file: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
    pub output_file: Option<String>,
    pub spring: bool,
    pub prefix: String,
    pub replacement_map: HashMap<char, String>,
    pub file: Option<String>,
}

impl Args {
    pub fn validate_and_convert(self) -> Result<Configuration, InternalError> {
        let mut errors: Vec<String> = Vec::new();
        if self.spring && !self.replacement.is_empty() {
            errors.push("replacements are not allowed when 'spring' flag is passed".to_string());
        }
        if self.prefix == "" {
            errors.push("prefix must not be empty".to_string());
        }
        // !self.spring || self.replacement.is_empty()
        if self.spring && errors.is_empty() {
            return Ok(Configuration {
                output_file: self.output_file,
                spring: self.spring,
                replacement_map: HashMap::new(),
                prefix: self.prefix,
                file: self.file,
            });
        }

        // !self.replacement.is_empty()
        let mut replacement_map: HashMap<char, String> = HashMap::new();
        fn error_msg<S: AsRef<str> + Display>(replacement: S, message: S) -> String {
            format!(
                "replacement '{}' does not contain valid mapping in the format 'c#str': {}",
                replacement, message
            )
        }
        for replacement in self.replacement {
            let replacement_for_errors = replacement.clone();
            if !replacement.contains("#") {
                errors.push(error_msg(replacement_for_errors.as_str(), "'#' missing"));
                continue;
            }
            let (left, right) = replacement.split_once("#").unwrap();
            let (left, right) = (left.trim(), right.trim());
            let left = if left == "\\-" { "-" } else { left };
            if left.len() > 1 {
                errors.push(error_msg(
                    replacement_for_errors,
                    format!("'{}' is not a character", left),
                ));
                continue;
            }
            let character: char = left.chars().next().unwrap();
            replacement_map.insert(character, right.to_string());
        }
        if !errors.is_empty() {
            return Err(InternalError::ArgumentValidationErrors(errors));
        }
        Ok(Configuration {
            output_file: self.output_file,
            spring: self.spring,
            replacement_map,
            prefix: self.prefix,
            file: self.file,
        })
    }
}

#[cfg(test)]
mod args_tests {
    #[cfg(test)]
    mod validate_and_covert_tests {
        use super::super::*;
        use crate::test_utils::assert_contains_exactly_in_any_order;

        fn assert_argument_validation_error(
            result: &Result<Configuration, InternalError>,
            messages: &Vec<String>,
        ) {
            match result {
                Ok(_) => panic!("result is successful, error expected"),
                Err(err) => match err {
                    InternalError::ArgumentValidationErrors(actual_messages) => {
                        assert_contains_exactly_in_any_order(actual_messages, messages)
                    }
                    _ => panic!("error is not ArgumentValidationError"),
                },
            }
        }
        #[test]
        fn should_be_invalid_if_spring_flag_true_and_replacement_present() {
            let args = Args {
                output_file: None,
                spring: true,
                prefix: "PREFIX_".to_string(),
                replacement: vec![".#_".to_string()],
                file: None,
            };

            assert_argument_validation_error(
                &args.validate_and_convert(),
                &vec!["replacements are not allowed when 'spring' flag is passed".to_string()],
            );
        }

        #[test]
        fn should_be_invalid_if_prefix_is_empty() {
            let args = Args {
                output_file: None,
                spring: true,
                prefix: "".to_string(),
                replacement: vec![],
                file: None,
            };

            assert_argument_validation_error(
                &args.validate_and_convert(),
                &vec!["prefix must not be empty".to_string()],
            );
        }

        #[test]
        fn should_return_configuration_with_spring_flag_passed() {
            let args = Args {
                output_file: Some("output2".to_string()),
                spring: true,
                prefix: "PREFIX_".to_string(),
                replacement: vec![],
                file: Some("file1".to_string()),
            };

            assert_eq!(
                args.validate_and_convert().unwrap(),
                Configuration {
                    output_file: Some("output2".to_string()),
                    spring: true,
                    prefix: "PREFIX_".to_string(),
                    replacement_map: HashMap::new(),
                    file: Some("file1".to_string()),
                }
            )
        }

        #[test]
        fn should_be_invalid_if_any_replacement_does_not_contain_arrow() {
            let args = Args {
                output_file: None,
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec!["invalid".to_string()],
                file: None,
            };

            assert_argument_validation_error(&args.validate_and_convert(),
                &vec!["replacement 'invalid' does not contain valid mapping in the format 'c#str': '#' missing".to_string()]
            );
        }

        #[test]
        fn should_be_invalid_if_any_replacement_does_not_have_character() {
            let args = Args {
                output_file: None,
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec!["asdf#str".to_string()],
                file: None,
            };

            assert_argument_validation_error(
                &args.validate_and_convert(),
                &vec!["replacement 'asdf#str' does not contain valid mapping in the format 'c#str': 'asdf' is not a character".to_string()]
            );
        }

        #[test]
        fn should_be_invalid_if_multiple_errors_present() {
            let args = Args {
                output_file: None,
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec!["invalid1".to_string(), "fdas#str".to_string()],
                file: None,
            };

            let result = args.validate_and_convert();
            match result {
                Err(InternalError::ArgumentValidationErrors(messages)) => {
                    assert_contains_exactly_in_any_order(messages, vec![
                        "replacement 'invalid1' does not contain valid mapping in the format 'c#str': '#' missing".to_string(),
                        "replacement 'fdas#str' does not contain valid mapping in the format 'c#str': 'fdas' is not a character".to_string()
                    ]);
                }
                _ => panic!(
                    "result from validate and convert is not an ArgumentValidationErrors instance"
                ),
            }
        }

        #[test]
        fn should_return_parsed_replacement_mapping() {
            let args = Args {
                output_file: None,
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec!["-#__".to_string(), ".#_".to_string()],
                file: None,
            };

            assert_eq!(
                args.validate_and_convert().unwrap(),
                Configuration {
                    output_file: None,
                    spring: false,
                    prefix: "PREFIX_".to_string(),
                    replacement_map: hashmap! {
                        '.' => "_".to_string(),
                        '-' => "__".to_string(),
                    },
                    file: None,
                }
            )
        }

        #[test]
        fn should_map_escaped_dash_as_dash() {
            let args = Args {
                output_file: None,
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec!["\\-#__".to_string(), ".#_".to_string()],
                file: None,
            };

            assert_eq!(
                args.validate_and_convert().unwrap(),
                Configuration {
                    output_file: None,
                    spring: false,
                    prefix: "PREFIX_".to_string(),
                    replacement_map: hashmap! {
                        '.' => "_".to_string(),
                        '-' => "__".to_string(),
                    },
                    file: None,
                }
            )
        }

        #[test]
        fn should_trim_spaces_in_replacement_mapping() {
            let args = Args {
                output_file: Some("foo".to_string()),
                spring: false,
                prefix: "PREFIX_".to_string(),
                replacement: vec![" - # __ ".to_string(), "  .  # _ ".to_string()],
                file: None,
            };

            assert_eq!(
                args.validate_and_convert().unwrap(),
                Configuration {
                    output_file: Some("foo".to_string()),
                    spring: false,
                    prefix: "PREFIX_".to_string(),
                    replacement_map: hashmap! {
                        '.' => "_".to_string(),
                        '-' => "__".to_string(),
                    },
                    file: None,
                }
            )
        }
    }
}
