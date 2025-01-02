use crate::model::{Property, InternalError};

pub fn parse_line(line: &str, line_num: i32) -> Result<Property, InternalError> {
    match line.split_once("=") {
        None => Err(InternalError::parse_error(line_num, "missing '='")),
        Some((key, value)) => {
            if key.contains(" ") {
                return Err(InternalError::parse_error(line_num, format!("key '{}' contains spaces", key).as_str()));
            }
            let value = value.trim_end_matches("\n");
            Ok(Property::new(key, value))
        },
    }
}

#[cfg(test)]
mod test {
    use crate::model::{InternalError, Property};
    use super::parse_line;

    const LINE_NUM: i32= 56;

    fn parse(s: &str) -> Result<Property, InternalError> {
        parse_line(s, LINE_NUM)
    }

    fn parse_error(message: &str) -> InternalError {
        InternalError::parse_error(LINE_NUM, message)
    }

    #[test]
    fn parse_line_should_fail_if_equals_not_present() {
        let l = parse("foobar");

        assert_eq!(l, Err(parse_error("missing '='")));
    }

    #[test]
    fn parse_line_should_separate_key_from_value() {
        let l = parse("key=value");

        assert_eq!(l, Ok(Property::new("key", "value")))
    }

    #[test]
    fn parse_line_should_strip_new_line_at_end_of_value() {
        let l = parse("key1=value1\n");

        assert_eq!(l, Ok(Property::new("key1", "value1")));
    }

    #[test]
    fn parse_line_should_strip_multiple_new_line_at_end_of_value() {
        let l = parse("key1=value1\n\n");

        assert_eq!(l, Ok(Property::new("key1", "value1")));
    }

    #[test]
    fn parse_line_should_fail_if_key_as_spaces() {
        let l = parse("  key  =foobar");

        assert_eq!(l, Err(parse_error("key '  key  ' contains spaces")));
    }

    #[test]
    fn parse_line_should_retain_spaces_in_value() {
        let l = parse("key=  bar foo   ");

        assert_eq!(l, Ok(Property::new("key", "  bar foo   ")));
    }
}