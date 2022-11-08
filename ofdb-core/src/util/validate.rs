use ofdb_entities::{
    address::Address,
    contact::Contact,
    event::Event,
    geo::{MapBbox, MapPoint},
    location::Location,
    place::Place,
};
use thiserror::Error;

pub use fast_chemail::is_valid_email;

pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}

pub trait AutoCorrect {
    fn auto_correct(self) -> Self;
}

const fn is_valid_license(s: &str) -> bool {
    // NOTE:
    // The actual license has to be checked
    // in the corresponding use case.
    !s.is_empty()
}

pub fn is_valid_bbox(bbox: &MapBbox) -> bool {
    bbox.is_valid() && !bbox.is_empty()
}

#[derive(Debug, Error)]
pub enum PlaceInvalidation {
    #[error("Invalid licence")]
    License,
    #[error(transparent)]
    Contact(ContactInvalidation),
}

impl Validate for Place {
    type Error = PlaceInvalidation;
    fn validate(&self) -> Result<(), Self::Error> {
        if !is_valid_license(&self.license) {
            return Err(Self::Error::License);
        }
        //TODO: check title
        self.contact
            .as_ref()
            .map(|c| c.validate())
            .transpose()
            .map_err(Self::Error::Contact)?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ContactInvalidation {
    #[error("Invalid email")]
    EmailAddress,
}

impl Validate for Contact {
    type Error = ContactInvalidation;
    fn validate(&self) -> Result<(), Self::Error> {
        // NOTE:
        // self.email should be always valid because
        // the validation is done in the constructor.
        if let Some(ref e) = self.email {
            if !is_valid_email(e.as_str()) {
                return Err(Self::Error::EmailAddress);
            }
        }
        //TODO: check phone
        Ok(())
    }
}

impl AutoCorrect for Event {
    fn auto_correct(mut self) -> Self {
        self.description = self.description.filter(|x| !x.is_empty());
        self.location = self.location.and_then(|l| {
            let l = l.auto_correct();
            if l.address.is_none() && l.pos == MapPoint::default() {
                None
            } else {
                Some(l)
            }
        });
        self.contact = self.contact.and_then(|c| {
            let c = c.auto_correct();
            if c.email.is_none() && c.phone.is_none() {
                None
            } else {
                Some(c)
            }
        });
        self.created_by = self.created_by.filter(|x| !x.as_str().is_empty());
        self
    }
}

#[derive(Debug, Error)]
pub enum EventInvalidation {
    #[error("Invalid title")]
    Title,
    #[error("The end date is before the start date")]
    EndDateBeforeStart,
    #[error(transparent)]
    Contact(ContactInvalidation),
}

impl Validate for Event {
    type Error = EventInvalidation;
    fn validate(&self) -> Result<(), Self::Error> {
        if self.title.is_empty() {
            return Err(Self::Error::Title);
        }
        if let Some(ref c) = self.contact {
            c.validate().map_err(Self::Error::Contact)?;
        }
        if let Some(end) = self.end {
            if end < self.start {
                return Err(Self::Error::EndDateBeforeStart);
            }
        }
        Ok(())
    }
}

impl AutoCorrect for Contact {
    fn auto_correct(mut self) -> Self {
        self.email = self.email.filter(|x| !x.as_str().is_empty());
        self.phone = self.phone.filter(|x| !x.is_empty());
        self
    }
}

impl AutoCorrect for Location {
    fn auto_correct(mut self) -> Self {
        self.address = self
            .address
            .map(AutoCorrect::auto_correct)
            .filter(|a| !a.is_empty());
        self
    }
}

impl AutoCorrect for Address {
    fn auto_correct(mut self) -> Self {
        self.street = self.street.filter(|x| !x.is_empty());
        self.zip = self.zip.filter(|x| !x.is_empty());
        self.city = self.city.filter(|x| !x.is_empty());
        self.country = self.country.filter(|x| !x.is_empty());
        self.state = self.state.filter(|x| !x.is_empty());
        self
    }
}

#[cfg(test)]
mod tests {
    use ofdb_entities::{email::EmailAddress, time::Timestamp};
    use time::Duration;

    use super::*;

    #[test]
    fn license_test() {
        assert!(!is_valid_license(""));
        assert!(is_valid_license("non-empty-string"));
    }

    #[test]
    fn email_test() {
        assert!(!is_valid_email("foo"));
        assert!(!is_valid_email("foo@bar"));
        assert!(is_valid_email("foo@bar.tld"));
    }

    #[test]
    fn contact_email_test() {
        assert!(Contact {
            name: None,
            email: Some(EmailAddress::new_unchecked("foo".to_string())),
            phone: None
        }
        .validate()
        .is_err());
        assert!(Contact {
            name: None,
            email: Some(EmailAddress::new_unchecked("foo@bar.tld".to_string())),
            phone: None
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn event_autocorrect() {
        let e = Event {
            id: "x".into(),
            title: "foo".into(),
            description: None,
            start: Timestamp::from_secs(0),
            end: None,
            location: None,
            contact: None,
            tags: vec![],
            homepage: None,
            created_by: None,
            registration: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        };

        let mut x = e.clone();
        x.description = Some("".to_string());
        assert!(x.auto_correct().description.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            name: None,
            email: Some(EmailAddress::new_unchecked("".to_string())),
            phone: None,
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            name: None,
            email: None,
            phone: Some("".into()),
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.created_by = Some(EmailAddress::new_unchecked("".to_string()));
        assert!(x.auto_correct().created_by.is_none());

        let mut x = e;
        x.location = Some(Location {
            pos: Default::default(),
            address: Some(Address {
                street: None,
                zip: None,
                city: Some("".into()),
                country: None,
                state: None,
            }),
        });
        assert!(x.auto_correct().location.is_none());
    }

    #[test]
    fn address_autocorrect() {
        let a = Address {
            street: None,
            zip: None,
            city: None,
            country: None,
            state: None,
        };

        let mut x = a.clone();
        x.street = Some("".to_string());
        assert!(x.auto_correct().street.is_none());

        let mut x = a.clone();
        x.zip = Some("".to_string());
        assert!(x.auto_correct().zip.is_none());

        let mut x = a.clone();
        x.city = Some("".to_string());
        assert!(x.auto_correct().city.is_none());

        let mut x = a.clone();
        x.country = Some("".to_string());
        assert!(x.auto_correct().country.is_none());

        let mut x = a;
        x.state = Some("".to_string());
        assert!(x.auto_correct().state.is_none());
    }

    #[test]
    fn location_autocorrect() {
        let l = Location {
            pos: Default::default(),
            address: Some(Address {
                street: None,
                zip: Some("".into()),
                city: None,
                country: None,
                state: None,
            }),
        };
        assert!(l.auto_correct().address.is_none());
    }

    #[test]
    fn validate_event_start() {
        let now = Timestamp::now();
        let e = Event {
            id: "x".into(),
            title: "foo".into(),
            description: None,
            start: now,
            end: None,
            location: None,
            contact: None,
            tags: vec![],
            homepage: None,
            created_by: None,
            registration: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        };
        assert!(e.validate().is_ok());
        assert!(Event {
            start: now - Duration::days(10_000),
            ..e.clone()
        }
        .validate()
        .is_ok());
        assert!(Event {
            start: now + Duration::days(10_000),
            ..e
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn event_with_invalid_end_test() {
        let e = Event {
            id: "x".into(),
            title: "foo".into(),
            description: None,
            start: Timestamp::from_secs(100),
            end: Some(Timestamp::from_secs(99)),
            location: None,
            contact: None,
            tags: vec![],
            homepage: None,
            created_by: None,
            registration: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        };
        assert!(e.validate().is_err());
    }

    #[test]
    fn bbox_test() {
        let p1 = MapPoint::from_lat_lng_deg(48.123, 5.123);
        let p2 = MapPoint::try_from_lat_lng_deg(48.123, 500.123).unwrap_or_default();
        let p3 = MapPoint::from_lat_lng_deg(49.123, 10.123);
        let valid_bbox = MapBbox::new(p1, p3);
        let empty_bbox = MapBbox::new(p3, p3);
        let invalid_bbox = MapBbox::new(p2, p3);
        assert!(is_valid_bbox(&valid_bbox));
        assert!(!is_valid_bbox(&empty_bbox));
        assert!(!is_valid_bbox(&invalid_bbox));
    }
}
