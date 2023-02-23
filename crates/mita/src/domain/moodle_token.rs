use eyre::Context;

/// A token consists of 32 characters in `a..f` or `0..9`.
///
/// This is stored as a 16-byte array.
#[derive(Debug)]
pub struct MoodleToken([u8; 16]);

impl std::str::FromStr for MoodleToken {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).wrap_err("moodle token is not a hex string")?;
        Ok(Self(
            (*bytes)
                .try_into()
                .wrap_err("moodle token has incorrect length")?,
        ))
    }
}

impl AsRef<[u8]> for MoodleToken {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use claims::{assert_err, assert_ok};
    use proptest::prelude::*;

    use super::MoodleToken;

    proptest! {
        #[test]
        fn doesnt_crash(s in "\\PC*") {
            drop(s.parse::<MoodleToken>());
        }
    }

    proptest! {
        #[test]
        fn valid_token(s in "[0-9a-fA-F]{32}") {
            assert_ok!(s.parse::<MoodleToken>());
        }
    }

    fn invalid_hex_token() -> impl Strategy<Value = String> {
        "\\PC{32}".prop_filter("invalid hex only", |s| hex::decode(s).is_err())
    }

    proptest! {
        #[test]
        fn error_on_invalid_hex_token(s in invalid_hex_token()) {
            let _ = assert_err!(s.parse::<MoodleToken>());
        }
    }

    fn wrong_length_hex_token() -> impl Strategy<Value = String> {
        any::<usize>()
            .prop_filter("!= 32", |&size| size != 32)
            .prop_map(|len| format!("[0-9a-fA-F]{{{len}}}"))
    }

    proptest! {
        #[test]
        fn error_on_wrong_length_hex_token(s in wrong_length_hex_token()) {
            let _ = assert_err!(s.parse::<MoodleToken>());
        }
    }
}
