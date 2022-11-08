use std::{fmt, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmailAddress {
    address: String,
    display_name: Option<String>,
}

impl EmailAddress {
    pub const fn new_unchecked(address: String) -> Self {
        Self {
            address,
            display_name: None,
        }
    }
    pub fn into_string(self) -> String {
        self.address
    }
    pub fn as_str(&self) -> &str {
        self.address.as_str()
    }
}

#[derive(Debug, Error)]
#[error("Invalid E-Mail address")]
pub struct EmailAddressParseError;

impl FromStr for EmailAddress {
    type Err = EmailAddressParseError;
    fn from_str(s: &str) -> Result<EmailAddress, Self::Err> {
        let info = mailparse::addrparse(s)
            .ok()
            .and_then(|list| list.extract_single_info())
            .ok_or(EmailAddressParseError)?;
        Ok(Self {
            address: info.addr,
            display_name: info.display_name,
        })
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let EmailAddress {
            address,
            display_name,
        } = self;
        if let Some(display_name) = &display_name {
            write!(
                f,
                r#""{display_name}" <{address}>"#,
                display_name = display_name.replace('"', r#"\""#)
            )
        } else {
            write!(f, "{address}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailContent {
    pub subject: String,
    pub body: String,
}
