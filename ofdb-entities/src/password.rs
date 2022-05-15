use std::{fmt, str::FromStr};

use pwhash::bcrypt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Password(String);

impl Password {
    pub const fn min_len() -> usize {
        6
    }

    pub fn verify(&self, password: &str) -> bool {
        pwhash::bcrypt::verify(password, &self.0)
    }
}

impl From<String> for Password {
    fn from(from: String) -> Self {
        Self(from)
    }
}

impl From<Password> for String {
    fn from(from: Password) -> Self {
        from.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum ParseError {
    InsufficientLength,
    Invalid,
}

impl FromStr for Password {
    type Err = ParseError;

    fn from_str(password: &str) -> Result<Self, Self::Err> {
        if password.len() < Password::min_len() {
            return Err(ParseError::InsufficientLength);
        }
        let res = Self(bcrypt::hash(password).map_err(|e| match e {
            pwhash::error::Error::InsufficientLength => ParseError::InsufficientLength,
            _ => ParseError::Invalid,
        })?);
        debug_assert!(res.verify(password));
        Ok(res)
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_hash_and_verify_password() {
        let input = "p^$$w%&7*{}";
        let password = input.parse::<Password>().unwrap();
        assert_ne!(password.as_ref(), input);
        assert!(password.verify(input));
    }

    #[test]
    fn should_fail_to_parse_short_passwords() {
        assert!("a".parse::<Password>().is_err());
        assert!("ab".parse::<Password>().is_err());
        assert!("abc".parse::<Password>().is_err());
        assert!("abcd".parse::<Password>().is_err());
        assert!("abcde".parse::<Password>().is_err());
    }
}
