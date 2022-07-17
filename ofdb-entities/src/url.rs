//! A replacement of [`url::Url`](https://docs.rs/url/2.2.0/url/struct.Url.html)
//! in order to reduce WASM file sizes.
//!
//! This hopefully will be obsolete as soon as
//! [rust-url #557](https://github.com/servo/rust-url/issues/557)
//! is resolved.

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url(String);

impl Url {
    // https://docs.rs/url/2.2.0/url/struct.Url.html#method.as_str
    pub fn as_str(&self) -> &str {
        &self.0
    }
    // https://docs.rs/url/2.2.0/url/struct.Url.html#method.into_string
    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub struct ParseError;

impl FromStr for Url {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // WARNING:
        // This ignores any checks!!
        // Use `url::Url::parse` to verify valid URLs
        Ok(Url(s.to_string()))
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Url> for String {
    fn from(url: Url) -> Self {
        url.0
    }
}
