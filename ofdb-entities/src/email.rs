use std::{borrow::Borrow, fmt, ops::Deref, str::FromStr};

// TODO: rename to EmailAddress
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Email(String);

impl AsRef<String> for Email {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<Email> for String {
    fn from(from: Email) -> Self {
        from.0
    }
}

impl From<String> for Email {
    fn from(from: String) -> Self {
        Self(from)
    }
}

impl From<&str> for Email {
    fn from(from: &str) -> Self {
        from.to_owned().into()
    }
}

impl FromStr for Email {
    type Err = ();
    fn from_str(s: &str) -> Result<Email, Self::Err> {
        Ok(s.into())
    }
}

impl Borrow<str> for Email {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl Deref for Email {
    type Target = String;

    fn deref(&self) -> &String {
        self.as_ref()
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.as_ref())
    }
}
