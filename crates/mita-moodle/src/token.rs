use std::ops::Deref;

use eyre::WrapErr;
use secrecy::Secret;

/// A token consists of 32 characters in `a..f` or `0..9`.
///
/// # Example
///
/// ```
/// use mita_moodle::MoodleToken;
/// use secrecy::ExposeSecret;
///
/// let secret = "a".repeat(32);
/// let token = secret.parse::<MoodleToken>().unwrap();
/// assert_eq!(token.expose_secret(), &secret);
/// ```
///

#[derive(Clone, Debug)]
pub struct MoodleToken(Secret<String>);

impl Deref for MoodleToken {
    type Target = Secret<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for MoodleToken {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).wrap_err("moodle token is not a hex string")?;
        <[u8; 16]>::try_from(bytes)
            .map_err(|_| eyre::eyre!("moodle token is not at correct length"))?;
        Ok(Self(Secret::new(s.to_string())))
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
