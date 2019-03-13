use crate::core::error::{Error, ParameterError};

use std::{fmt, ops::Deref, str::FromStr};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nonce(Uuid);

impl Nonce {
    pub const STR_LEN: usize = 32;

    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for Nonce {
    fn from(from: Uuid) -> Self {
        Self(from)
    }
}

impl From<Nonce> for Uuid {
    fn from(from: Nonce) -> Self {
        from.0
    }
}

impl AsRef<Uuid> for Nonce {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl Deref for Nonce {
    type Target = Uuid;

    fn deref(&self) -> &Uuid {
        &self.0
    }
}

impl FromStr for Nonce {
    type Err = Error;

    fn from_str(nonce_str: &str) -> Result<Self, Self::Err> {
        nonce_str
            .parse::<Uuid>()
            .map(Into::into)
            .map_err(|_| Error::Parameter(ParameterError::InvalidNonce))
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0.to_simple_ref().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_unique_instances() {
        let n1 = Nonce::new();
        let n2 = Nonce::new();
        assert_ne!(n1, n2);
    }

    #[test]
    fn should_convert_from_to_string() {
        let n1 = Nonce::new();
        let s1 = n1.to_string();
        assert_eq!(Nonce::STR_LEN, s1.len());
        let n2 = s1.parse::<Nonce>().unwrap();
        assert_eq!(n1, n2);
        assert_eq!(s1, n2.to_string());
    }
}
