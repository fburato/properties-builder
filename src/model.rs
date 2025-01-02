use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct Property {
    key: String,
    value: String,
}

impl Property {
    pub fn new<S: AsRef<str>>(key: S, value: S) -> Property {
        Property {
            key: key.as_ref().to_string(),
            value: value.as_ref().to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InternalError {
    ParseError { line_num: i32, message: String },
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
        }
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
        fn eq_should_be_well_implemented_for_parse_error() {
            let e1 = InternalError::parse_error(23, "foo");
            let e2 = InternalError::parse_error(23, "foo");
            let e3 = InternalError::parse_error(55, "foo");
            let e4 = InternalError::parse_error(23, "bar");

            assert_eq!(e1, e2);
            assert_ne!(e1, e3);
            assert_ne!(e1, e4);

            assert_eq!(e2, e1);
            assert_ne!(e3, e1);
            assert_ne!(e4, e1);
        }
    }
}
