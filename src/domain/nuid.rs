use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Nuid(String);

impl std::fmt::Display for Nuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Nuid {
    pub fn parse(s: &str) -> Result<Nuid, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 9;

        let all_integers = s.chars().all(|c| c.is_ascii_digit());

        if is_empty_or_whitespace || is_too_long || !all_integers {
            Err(format!("Invalid NUID! Given: {}", s))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl AsRef<str> for Nuid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::Nuid;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_9_grapheme_long_all_int_nuid_is_valid() {
        assert_ok!(Nuid::parse("0".repeat(9).as_str()));
    }

    #[test]
    fn whitespace_only_is_rejected() {
        assert_err!(Nuid::parse(" "));
    }

    #[test]
    fn empty_string_is_rejected() {
        assert_err!(Nuid::parse(""));
    }

    #[test]
    fn a_10_grapheme_long_all_int_nuid_is_rejected() {
        assert_err!(Nuid::parse("0".repeat(10).as_str()));
    }

    #[test]
    fn a_9_grapheme_long_all_string_nuid_is_rejected() {
        assert_err!(Nuid::parse("a".repeat(9).as_str()));
    }

    #[test]
    fn a_9_grapheme_long_string_with_1_to_8_ints_is_rejected() {
        let characters = ['0', 'a'];

        for num_a in 1..=8 {
            let permutation = vec!['a'; num_a];
            let permutation_string = permutation.iter().collect::<String>();
            let full_string = format!("{}{}", permutation_string, &"00000000"[..8 - num_a]);

            for i in 0..9 {
                for char in &characters {
                    let mut test_string = full_string.clone();
                    test_string.insert(i, *char);
                    assert_err!(
                        Nuid::parse(test_string.as_str()),
                        "The call to Nuid parse should have failed with the string: {}",
                        test_string
                    );
                }
            }
        }
    }
}
