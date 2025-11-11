use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        let contains_forbidden_charactes = s.chars().any(|g| forbidden_characters.contains(&g));
        if is_empty_or_whitespace || is_too_long || contains_forbidden_charactes {
            Err(format!("{s} is not a valid name"))
        } else {
            Ok(Self(s))
        }
    }
}

//we'll be implementing Asref for SubscriberName
//<T: AsRef<str>>(s: T)
//
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use claim::{assert_err, assert_ok};

    use crate::domain::*;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "i".repeat(255);
        assert_ok!(SubscriberName::parse(name));
    }
    #[test]
    fn a_grapheme_length_constraint_holds() {
        let name = "i".repeat(259);
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn empty_names_are_rejected() {
        let name = "  ".to_string();
        assert_err!(SubscriberName::parse(name));
    }
    #[test]
    fn unallowed_chars() {
        let name = "{idr(do something) [random]\\dlqson }".to_string();
        assert_err!(SubscriberName::parse(name));
    }
}
