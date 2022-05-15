use crate::time::*;
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

#[derive(Debug)]
pub struct NonceParseError;

impl fmt::Display for NonceParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Invalid Nonce")
    }
}

impl FromStr for Nonce {
    type Err = NonceParseError;

    fn from_str(nonce_str: &str) -> Result<Self, Self::Err> {
        nonce_str
            .parse::<Uuid>()
            .map(Into::into)
            .map_err(|_| NonceParseError)
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0.as_simple())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EmailNonce {
    pub email: String,
    pub nonce: Nonce,
}

pub type ActualTokenLen = usize;
pub type NonceString = String;

#[derive(Debug)]
pub enum EmailNonceDecodingError {
    Bs58(bs58::decode::Error),
    Utf8(std::string::FromUtf8Error),
    TooShort(ActualTokenLen),
    Parse(NonceString, NonceParseError),
}

impl EmailNonce {
    pub fn encode_to_string(&self) -> String {
        let nonce = self.nonce.to_string();
        debug_assert_eq!(Nonce::STR_LEN, nonce.len());
        let mut concat = String::with_capacity(self.email.len() + nonce.len());
        concat += &self.email;
        concat += &nonce;
        bs58::encode(concat).into_string()
    }

    pub fn decode_from_str(encoded: &str) -> Result<EmailNonce, EmailNonceDecodingError> {
        let decoded = bs58::decode(encoded)
            .into_vec()
            .map_err(EmailNonceDecodingError::Bs58)?;
        let mut concat = String::from_utf8(decoded).map_err(EmailNonceDecodingError::Utf8)?;
        if concat.len() < Nonce::STR_LEN {
            return Err(EmailNonceDecodingError::TooShort(concat.len()));
        }
        let email_len = concat.len() - Nonce::STR_LEN;
        let nonce_slice: &str = &concat[email_len..];
        let nonce = nonce_slice
            .parse::<Nonce>()
            .map_err(|err| EmailNonceDecodingError::Parse(nonce_slice.into(), err))?;
        concat.truncate(email_len);
        let email = concat;
        Ok(Self { email, nonce })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UserToken {
    pub email_nonce: EmailNonce,
    // TODO: Convert time stamps from second to millisecond precision?
    pub expires_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_email_nonce() {
        let example = EmailNonce {
            email: "test@example.com".into(),
            nonce: Nonce::new(),
        };
        let encoded = example.encode_to_string();
        let decoded = EmailNonce::decode_from_str(&encoded).unwrap();
        assert_eq!(example, decoded);
    }

    #[test]
    fn encode_decode_email_nonce_with_empty_email() {
        let example = EmailNonce {
            email: "".into(),
            nonce: Nonce::new(),
        };
        let encoded = example.encode_to_string();
        let decoded = EmailNonce::decode_from_str(&encoded).unwrap();
        assert_eq!(example, decoded);
    }

    #[test]
    fn decode_empty_email_nonce() {
        assert!(EmailNonce::decode_from_str("").is_err());
    }

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
