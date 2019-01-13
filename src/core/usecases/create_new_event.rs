use super::create_user_from_email;
use crate::core::{
    prelude::*,
    util::{
        parse::parse_url_param,
        validate::{AutoCorrect, Validate},
    },
};
use std::str::FromStr;
use uuid::Uuid;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Deserialize, Debug, Clone)]
pub struct NewEvent {
    pub title        : String,
    pub description  : Option<String>,
    pub start        : u64,
    pub end          : Option<u64>,
    pub lat          : Option<f64>,
    pub lng          : Option<f64>,
    pub street       : Option<String>,
    pub zip          : Option<String>,
    pub city         : Option<String>,
    pub country      : Option<String>,
    pub email        : Option<String>,
    pub telephone    : Option<String>,
    pub homepage     : Option<String>,
    pub tags         : Option<Vec<String>>,
    pub created_by   : Option<String>,
    pub token        : Option<String>,
    pub registration : Option<String>,
}

// TODO: move this into an adapter
impl FromStr for RegistrationType {
    type Err = Error;
    fn from_str(s: &str) -> Result<RegistrationType> {
        match &*s.to_lowercase() {
            "email" => Ok(RegistrationType::Email),
            "telephone" => Ok(RegistrationType::Phone),
            "homepage" => Ok(RegistrationType::Homepage),
            _ => Err(ParameterError::RegistrationType.into()),
        }
    }
}

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

pub fn try_into_new_event<D: Db>(db: &mut D, e: NewEvent) -> Result<Event> {
    let NewEvent {
        title,
        description,
        start,
        end,
        email,
        telephone,
        lat,
        lng,
        street,
        zip,
        city,
        country,
        tags,
        created_by,
        registration,
        token,
        ..
    } = e;
    let org = if let Some(ref token) = token {
        let org = db.get_org_by_api_token(token).map_err(|e| match e {
            RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
            _ => Error::Repo(e),
        })?;
        Some(org)
    } else {
        None
    };
    let mut tags: Vec<_> = tags
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|t| t.replace("#", ""))
        .collect();
    tags.dedup();
    let owned_tags = db.get_all_tags_owned_by_orgs()?;
    for t in &tags {
        if owned_tags.iter().any(|id| id == t) {
            match org {
                Some(ref o) => {
                    if !o.owned_tags.iter().any(|x| x == t) {
                        return Err(ParameterError::OwnedTag.into());
                    }
                }
                None => {
                    return Err(ParameterError::OwnedTag.into());
                }
            }
        }
    }
    let address = if street.is_some() || zip.is_some() || city.is_some() || country.is_some() {
        Some(Address {
            street,
            zip,
            city,
            country,
        })
    } else {
        None
    };

    let location = if lat.is_some() || lng.is_some() || address.is_some() {
        Some(Location {
            lat: lat.unwrap_or(0.0),
            lng: lng.unwrap_or(0.0),
            address,
        })
    } else {
        None
    };
    let contact = if email.is_some() || telephone.is_some() {
        Some(Contact { email, telephone })
    } else {
        None
    };
    let id = Uuid::new_v4().to_simple_ref().to_string();
    let homepage = e
        .homepage
        .filter(|h| !h.is_empty())
        .map(|ref url| parse_url_param(url))
        .transpose()?;

    let created_by = if let Some(ref email) = created_by {
        let username = create_user_from_email(db, email)?;
        Some(username)
    } else {
        // NOTE: At the moment we require an email address:
        return Err(ParameterError::CreatorEmail.into());
        // But in the future we might allow anonymous creators:
        // None
    };

    let registration = match registration {
        Some(r) => {
            if r.is_empty() {
                None
            } else {
                let r = RegistrationType::from_str(&r)?;
                //TODO: move to validation
                match r {
                    RegistrationType::Email => match contact {
                        None => {
                            return Err(ParameterError::Contact.into());
                        }
                        Some(ref c) => {
                            if c.email.is_none() {
                                return Err(ParameterError::Email.into());
                            }
                        }
                    },
                    RegistrationType::Phone => match contact {
                        None => {
                            return Err(ParameterError::Contact.into());
                        }
                        Some(ref c) => {
                            if c.telephone.is_none() {
                                return Err(ParameterError::Phone.into());
                            }
                        }
                    },
                    RegistrationType::Homepage => {
                        if homepage.is_none() {
                            return Err(ParameterError::Url.into());
                        }
                    }
                }
                Some(r)
            }
        }
        None => None,
    };

    let event = Event {
        id,
        title,
        start,
        end,
        description,
        location,
        contact,
        homepage,
        tags,
        created_by,
        registration,
    };
    let event = event.auto_correct();
    event.validate()?;
    for t in &event.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    Ok(event)
}

pub fn create_new_event<D: Db>(db: &mut D, e: NewEvent) -> Result<String> {
    let new_event = try_into_new_event(db, e)?;
    let new_id = new_event.id.clone();

    debug!("Creating new event: {:?}", new_event);
    db.create_event(new_event)?;
    Ok(new_id)
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
    use uuid::Uuid;

    #[test]
    fn create_new_valid_event() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : 9999,
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : Some(vec!["foo".into(),"bar".into()]),
            created_by   : Some("foo@bar.com".into()),
            token        : None,
            registration : None,
        };
        let mut mock_db = MockDb::new();
        let id = create_new_event(&mut mock_db, x).unwrap();
        assert!(Uuid::parse_str(&id).is_ok());
        assert_eq!(mock_db.events.len(), 1);
        assert_eq!(mock_db.tags.len(), 2);
        let x = &mock_db.events[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.start, 9999);
        assert!(x.location.is_none());
        assert_eq!(x.description.as_ref().unwrap(), "bar");
        assert!(Uuid::parse_str(&x.id).is_ok());
        assert_eq!(x.id, id);
    }

    #[test]
    fn create_event_with_invalid_email() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : 9999,
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            email        : Some("fooo-not-ok".into()),
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : None,
            token        : None,
            registration : None,
        };
        let mut mock_db: MockDb = MockDb::new();
        assert!(create_new_event(&mut mock_db, x).is_err());
    }

    #[test]
    fn create_event_with_valid_non_existing_creator_email() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : 9999,
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : Some("fooo@bar.tld".into()),
            token        : None,
            registration : None,
        };
        let mut mock_db: MockDb = MockDb::new();
        assert!(create_new_event(&mut mock_db, x).is_ok());
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(&users[0].email, "fooo@bar.tld");
        assert_eq!(&users[0].username, "fooobartld");
    }

    #[test]
    fn create_event_with_valid_existing_creator_email() {
        let mut mock_db: MockDb = MockDb::new();
        mock_db
            .create_user(User {
                id: "x".into(),
                username: "foo".into(),
                email: "fooo@bar.tld".into(),
                password: "secret".into(),
                email_confirmed: true,
                role: Role::User,
            })
            .unwrap();
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : 9999,
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : Some("fooo@bar.tld".into()),
            token        : None,
            registration : None,
        };
        assert!(create_new_event(&mut mock_db, x).is_ok());
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
    }
}
