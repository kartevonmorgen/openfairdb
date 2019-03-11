use super::super::{
    entities::*,
    error::ParameterError,
    usecases::create_new_user::MAX_USERNAME_LEN,
    util::geo::{MapBbox, MapPoint},
};
use fast_chemail::is_valid_email;
use regex::Regex;
use url::Url;

lazy_static! {
    static ref USERNAME_REGEX: Regex =
        Regex::new(&format!("^[a-z0-9]{{1,{}}}$", MAX_USERNAME_LEN)).unwrap();
}

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

fn homepage(url: &str) -> Result<(), ParameterError> {
    Url::parse(url).map_err(|_| ParameterError::Url).map(|_| ())
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

pub fn username(name: &str) -> Result<(), ParameterError> {
    if !USERNAME_REGEX.is_match(name) {
        return Err(ParameterError::UserName);
    }
    Ok(())
}

impl Validate for Entry {
    fn validate(&self) -> Result<(), ParameterError> {
        //TODO: check title
        self.license
            .clone()
            .ok_or(ParameterError::License)
            .and_then(|ref l| license(l))?;

        if let Some(ref c) = self.contact {
            if let Some(ref e) = c.email {
                email(e)?;
            }
        }

        if let Some(ref h) = self.homepage {
            homepage(h)?;
        }

        Ok(())
    }
}

impl Validate for Contact {
    fn validate(&self) -> Result<(), ParameterError> {
        if let Some(ref e) = self.email {
            email(e)?;
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
        self.homepage = self.homepage.filter(|x| !x.is_empty());
        self.contact = self.contact.and_then(|c| {
            let c = c.auto_correct();
            if c.email.is_none() && c.telephone.is_none() {
                None
            } else {
                Some(c)
            }
        });
        self.created_by = self.created_by.filter(|x| !x.is_empty());
        self
    }
}

impl Validate for Event {
    fn validate(&self) -> Result<(), ParameterError> {
        if self.title.is_empty() {
            return Err(ParameterError::Title);
        }
        if let Some(ref c) = self.contact {
            c.validate()?;
        }
        if let Some(ref h) = self.homepage {
            homepage(h)?;
        }
        if let Some(end) = self.end {
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
        self.telephone = self.telephone.filter(|x| !x.is_empty());
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
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

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
    fn username_test() {
        assert!(username("").is_err());
        assert!(username("no-dash").is_err());
        assert!(username("foo").is_ok());
        assert!(username(&["x"; 40].join("")).is_ok());
        assert!(username(&["x"; 41].join("")).is_err());
    }

    #[test]
    fn homepage_test() {
        assert!(homepage("https://openfairdb.org").is_ok());
        assert!(homepage("openfairdb.org/foo").is_err());
    }

    #[test]
    fn contact_email_test() {
        assert!(Contact {
            email: Some("foo".into()),
            telephone: None
        }
        .validate()
        .is_err());
        assert!(Contact {
            email: Some("foo@bar.tld".into()),
            telephone: None
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
        };

        let mut x = e.clone();
        x.description = Some("".to_string());
        assert!(x.auto_correct().description.is_none());

        let mut x = e.clone();
        x.homepage = Some("".to_string());
        assert!(x.auto_correct().homepage.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            email: Some("".into()),
            telephone: None,
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.contact = Some(Contact {
            email: None,
            telephone: Some("".into()),
        });
        assert!(x.auto_correct().contact.is_none());

        let mut x = e.clone();
        x.created_by = Some("".to_string());
        assert!(x.auto_correct().created_by.is_none());

        let mut x = e.clone();
        x.location = Some(Location {
            pos: Default::default(),
            address: Some(Address {
                street: None,
                zip: None,
                city: Some("".into()),
                country: None,
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
            }),
        };
        assert!(l.auto_correct().address.is_none());
    }

    #[test]
    fn event_test() {
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
        };
        assert!(e.validate().is_ok());
    }

    #[test]
    fn event_with_invalid_homepage_test() {
        let e = Event {
            id: "x".into(),
            title: "foo".into(),
            description: None,
            start: NaiveDateTime::from_timestamp(0, 0),
            end: None,
            location: None,
            contact: None,
            tags: vec![],
            homepage: Some("bla".into()),
            created_by: None,
            registration: None,
            organizer: None,
            archived: None,
        };
        assert!(e.validate().is_err());
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
