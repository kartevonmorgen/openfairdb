use crate::{contact::*, id::*, location::*, time::*};
use chrono::prelude::*;
use std::str::FromStr;
use url::Url;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegistrationType {
    Email,
    Phone,
    Homepage,
}

#[derive(Debug)]
pub struct RegistrationTypeParseError;

impl FromStr for RegistrationType {
    type Err = RegistrationTypeParseError;
    fn from_str(s: &str) -> Result<RegistrationType, Self::Err> {
        match &*s.to_lowercase() {
            "email" => Ok(RegistrationType::Email),
            "telephone" => Ok(RegistrationType::Phone),
            "homepage" => Ok(RegistrationType::Homepage),
            _ => Err(RegistrationTypeParseError),
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id           : Id,
    pub title        : String,
    pub description  : Option<String>,
    // Both start/end time stamps are stored with second precision!
    pub start        : NaiveDateTime,
    pub end          : Option<NaiveDateTime>,
    pub location     : Option<Location>,
    pub contact      : Option<Contact>,
    pub tags         : Vec<String>,
    pub homepage     : Option<Url>,
    pub created_by   : Option<String>,
    pub registration : Option<RegistrationType>,
    pub organizer    : Option<String>,
    // TODO: Switch archived time stamp to millisecond precision?
    pub archived     : Option<Timestamp>,
    pub image_url     : Option<Url>,
    pub image_link_url: Option<Url>,
}

impl Event {
    pub fn strip_activity_details(self) -> Self {
        Self {
            created_by: None,
            ..self
        }
    }

    pub fn strip_contact_details(self) -> Self {
        Self {
            contact: None,
            ..self
        }
    }

    pub fn is_owned<'a>(&self, owned_tags: impl IntoIterator<Item = &'a str>) -> bool {
        // Exclusive ownership of events is determined by the associated tags
        owned_tags
            .into_iter()
            .any(|owned_tag| self.tags.iter().any(|tag| tag == owned_tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_type_from_str() {
        assert_eq!(
            RegistrationType::from_str("email").unwrap(),
            RegistrationType::Email
        );
        assert_eq!(
            RegistrationType::from_str("eMail").unwrap(),
            RegistrationType::Email
        );
        assert_eq!(
            RegistrationType::from_str("telephone").unwrap(),
            RegistrationType::Phone
        );
        assert_eq!(
            RegistrationType::from_str("Telephone").unwrap(),
            RegistrationType::Phone
        );
        assert_eq!(
            RegistrationType::from_str("homepage").unwrap(),
            RegistrationType::Homepage
        );
        assert_eq!(
            RegistrationType::from_str("Homepage").unwrap(),
            RegistrationType::Homepage
        );
        assert!(RegistrationType::from_str("foo").is_err());
        assert!(RegistrationType::from_str("").is_err());
    }
}
