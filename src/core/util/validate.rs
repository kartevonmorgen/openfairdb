use super::super::{
    entities::*,
    error::ParameterError,
    util::geo::{MapBbox, MapPoint},
};
use chrono::{prelude::*, Duration};
use fast_chemail::is_valid_email;

pub trait Validate {
    fn validate(&self) -> Result<(), ParameterError>;
}

pub trait AutoCorrect {
    fn auto_correct(self) -> Self;
}

pub fn email(email: &str) -> Result<(), ParameterError> {
    if !is_valid_email(email) {
        return Err(ParameterError::Email);
    }
    Ok(())
}

fn license(s: &str) -> Result<(), ParameterError> {
    match s {
        "CC0-1.0" | "ODbL-1.0" => Ok(()),
        _ => Err(ParameterError::License),
    }
}

pub fn bbox(bbox: &MapBbox) -> Result<(), ParameterError> {
    if !bbox.is_valid() || bbox.is_empty() {
        return Err(ParameterError::Bbox);
    }
    Ok(())
}

impl Validate for Place {
    fn validate(&self) -> Result<(), ParameterError> {
        license(&self.license)?;

        //TODO: check title
        self.contact.as_ref().map(|c| c.validate()).transpose()?;

        Ok(())
    }
}

impl Validate for Contact {
    fn validate(&self) -> Result<(), ParameterError> {
        if let Some(ref e) = self.email {
            email(e.as_ref())?;
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
        self.created_by = self.created_by.filter(|x| !x.is_empty());
        self
    }
}

fn min_valid_event_date_time(now: NaiveDateTime) -> NaiveDateTime {
    // Allow edits up to 7 days = 1 week in the past to compensate for
    // different time zones (up to 1 day) and for testing
    now - Duration::from_std(std::time::Duration::from_secs(7 * 24 * 60 * 60)).unwrap()
}

fn max_valid_event_date_time(now: NaiveDateTime) -> NaiveDateTime {
    // Allow only up to 100 years in the future to distinguish invalid
    // millisecond timestamps from valid second timestamps
    now + Duration::from_std(std::time::Duration::from_secs(100 * 365 * 24 * 60 * 60)).unwrap()
}

impl Validate for Event {
    fn validate(&self) -> Result<(), ParameterError> {
        if self.title.is_empty() {
            return Err(ParameterError::Title);
        }
        if let Some(ref c) = self.contact {
            c.validate()?;
        }
        let now = Utc::now().naive_utc();
        let min_since = min_valid_event_date_time(now);
        let max_until = max_valid_event_date_time(now);
        if self.start < min_since || self.start > max_until {
            return Err(ParameterError::DateTimeOutOfRange);
        }
        if let Some(end) = self.end {
            if end < min_since || end > max_until {
                return Err(ParameterError::DateTimeOutOfRange);
            }
            if end < self.start {
                return Err(ParameterError::EndDateBeforeStart);
            }
        }
        Ok(())
    }
}

impl AutoCorrect for Contact {
    fn auto_correct(mut self) -> Self {
        self.email = self.email.filter(|x| !x.is_empty());
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
    use super::*;

    #[test]
    fn license_test() {
        assert!(license("CC0-1.0").is_ok());
        assert!(license("CC0").is_err());
        assert!(license("ODbL-1.0").is_ok());
    }

    #[test]
    fn email_test() {
        assert!(email("foo").is_err());
        assert!(email("foo@bar").is_err());
        assert!(email("foo@bar.tld").is_ok());
    }

    #[test]
    fn contact_email_test() {
        assert!(Contact {
            email: Some("foo".into()),
            phone: None
        }
        .validate()
        .is_err());
        assert!(Contact {
            email: Some("foo@bar.tld".into()),
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
            start: NaiveDateTime::from_timestamp(0, 0),
            end: None,
            location: None,
            contact: None,
            tags: vec![],
            homepage: None,
            created_by: None,
            registration: None,
            organizer: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        };

        let mut x = e.clone();
        x.description = Some("".to_string());
        assert!(x.auto_correct().description.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            email: Some("".into()),
            phone: None,
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            email: None,
            phone: Some("".into()),
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.created_by = Some("".to_string());
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
        let now = Utc::now().naive_utc();
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
            organizer: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        };
        assert!(e.validate().is_ok());
        assert!(Event {
            start: max_valid_event_date_time(now),
            ..e.clone()
        }
        .validate()
        .is_ok());
        assert!(Event {
            start: max_valid_event_date_time(now) + Duration::seconds(10),
            ..e.clone()
        }
        .validate()
        .is_err());
        assert!(Event {
            start: min_valid_event_date_time(now),
            ..e.clone()
        }
        .validate()
        .is_err());
        assert!(Event {
            start: min_valid_event_date_time(now) + Duration::seconds(10),
            ..e.clone()
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
            start: NaiveDateTime::from_timestamp(100, 0),
            end: Some(NaiveDateTime::from_timestamp(99, 0)),
            location: None,
            contact: None,
            tags: vec![],
            homepage: None,
            created_by: None,
            registration: None,
            organizer: None,
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
        assert!(bbox(&valid_bbox).is_ok());
        assert!(bbox(&empty_bbox).is_err());
        assert!(bbox(&invalid_bbox).is_err());
    }
}
