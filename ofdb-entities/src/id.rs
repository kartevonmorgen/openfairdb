use std::{borrow::Borrow, fmt, str::FromStr};

use uuid::Uuid;

/// Portable public identifier with a string representation.
// TODO: use `Uuid` and derive `Hash`
#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Id(String);

impl Id {
    pub fn new() -> Self {
        Uuid::new_v4().into()
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<String> for Id {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Id {
    fn from(from: String) -> Self {
        Self(from)
    }
}

impl From<&str> for Id {
    fn from(from: &str) -> Self {
        from.to_owned().into()
    }
}

impl From<Uuid> for Id {
    fn from(from: Uuid) -> Self {
        from.as_simple().to_string().into()
    }
}

impl From<Id> for String {
    fn from(from: Id) -> Self {
        from.0
    }
}

impl FromStr for Id {
    type Err = ();
    fn from_str(s: &str) -> Result<Id, Self::Err> {
        Ok(s.into())
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.as_ref())
    }
}
