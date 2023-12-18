use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ApplicantName(String);

impl ApplicantName {
    pub fn parse(s: &str) -> Result<ApplicantName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("Invalid name! Given: {}", s))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl AsRef<str> for ApplicantName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::ApplicantName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapeme_long_name_is_valid() {
        assert_ok!(ApplicantName::parse("a".repeat(256).as_str()));
    }

    #[test]
    fn a_name_longer_than_256_grapehemes_is_rejected() {
        assert_err!(ApplicantName::parse("a".repeat(257).as_str()));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        assert_err!(ApplicantName::parse(" "));
    }

    #[test]
    fn empty_string_is_rejected() {
        assert_err!(ApplicantName::parse(""));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            assert_err!(ApplicantName::parse(&name.to_string()));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        assert_ok!(ApplicantName::parse("Muneer Lalji"));
    }
}
