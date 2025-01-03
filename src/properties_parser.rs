use crate::model::{InternalError, Property};
use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Line {
    Ignorable(String),
    Prop(Property),
}

pub fn parse_line(line: &str, line_num: i32) -> Result<Line, InternalError> {
    if line.starts_with("#") {
        return Ok(Line::Ignorable(line.trim_end_matches("\n").to_string()));
    }
    let empty_line = Regex::new(r"^\s*\n*$").unwrap();
    if empty_line.is_match(line) {
        return Ok(Line::Ignorable(line.trim_end_matches("\n").to_string()));
    }
    match line.split_once("=") {
        None => Err(InternalError::parse_error(line_num, "missing '='")),
        Some((key, value)) => {
            if key.contains(" ") {
                return Err(InternalError::parse_error(
                    line_num,
                    format!("key '{}' contains spaces", key).as_str(),
                ));
            }
            let value = value.trim_end_matches("\n");
            Ok(Line::Prop(Property::new(key, value)))
        }
    }
}

#[cfg(test)]
mod parse_line_tests {
    use super::*;
    use crate::model::{InternalError, Property};

    const LINE_NUM: i32 = 56;

    fn parse(s: &str) -> Result<Line, InternalError> {
        parse_line(s, LINE_NUM)
    }

    fn assert_parse_error_with_message(
        result: &Result<Line, InternalError>,
        expected_message: &str,
    ) {
        match result {
            Ok(_) => panic!("result is OK, should be parse error"),
            Err(err) => match err {
                InternalError::ParseError {
                    line_num: _,
                    message,
                } => assert_eq!(message, expected_message),
                _ => panic!("result is not ParseError"),
            },
        }
    }

    #[test]
    fn should_fail_if_equals_not_present() {
        let l = parse("foobar");

        assert_parse_error_with_message(&l, "missing '='");
    }

    #[test]
    fn should_separate_key_from_value() {
        let l = parse("key=value");

        assert_eq!(l.unwrap(), Line::Prop(Property::new("key", "value")));
    }

    #[test]
    fn should_strip_new_line_at_end_of_value() {
        let l = parse("key1=value1\n");

        assert_eq!(l.unwrap(), Line::Prop(Property::new("key1", "value1")));
    }

    #[test]
    fn should_strip_multiple_new_line_at_end_of_value() {
        let l = parse("key1=value1\n\n");

        assert_eq!(l.unwrap(), Line::Prop(Property::new("key1", "value1")));
    }

    #[test]
    fn should_fail_if_key_as_spaces() {
        let l = parse("  key  =foobar");

        assert_parse_error_with_message(&l, "key '  key  ' contains spaces");
    }

    #[test]
    fn should_retain_spaces_in_value() {
        let l = parse("key=  bar foo   ");

        assert_eq!(l.unwrap(), Line::Prop(Property::new("key", "  bar foo   ")));
    }

    #[test]
    fn should_ignore_empty_lines_and_trim_newlines() {
        let l = parse("\n\n");

        assert_eq!(l.unwrap(), Line::Ignorable("".to_string()));
    }

    #[test]
    fn should_ignore_lines_with_spaces_and_tabs_trim_newlines() {
        let l = parse("  \t  \n\n");

        assert_eq!(l.unwrap(), Line::Ignorable("  \t  ".to_string()));
    }

    #[test]
    fn should_ignore_comment_lines() {
        let l = parse("# abc\n\n");

        assert_eq!(l.unwrap(), Line::Ignorable("# abc".to_string()));
    }
}
