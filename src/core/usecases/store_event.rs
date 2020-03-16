use super::create_user_from_email;
use crate::core::{
    prelude::*,
    util::{
        parse::parse_url_param,
        validate::{AutoCorrect, Validate},
    },
};
use chrono::prelude::*;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Deserialize, Default, Debug, Clone)]
pub struct NewEvent {
    pub title        : String,
    pub description  : Option<String>,
    pub start        : i64,
    pub end          : Option<i64>,
    pub lat          : Option<f64>,
    pub lng          : Option<f64>,
    pub street       : Option<String>,
    pub zip          : Option<String>,
    pub city         : Option<String>,
    pub country      : Option<String>,
    pub state        : Option<String>,
    pub email        : Option<String>,
    pub telephone    : Option<String>,
    pub homepage     : Option<String>,
    pub tags         : Option<Vec<String>>,
    pub created_by   : Option<String>,
    pub registration : Option<String>,
    pub organizer    : Option<String>,
    pub image_url     : Option<String>,
    pub image_link_url: Option<String>,
}

pub enum NewEventMode<'a> {
    Create,
    Update(&'a str),
}

#[derive(Debug, Clone)]
pub struct Storable(Event);

pub fn import_new_event<D: Db>(
    db: &D,
    token: Option<&str>,
    e: NewEvent,
    mode: NewEventMode,
) -> Result<Storable> {
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
        state,
        tags,
        created_by,
        registration,
        organizer,
        homepage,
        image_url,
        image_link_url,
        ..
    } = e;
    let org = token
        .map(|t| {
            db.get_org_by_api_token(t).map_err(|e| {
                log::warn!("Unknown or invalid API token: {}", t);
                match e {
                    RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
                    _ => Error::Repo(e),
                }
            })
        })
        .transpose()?;
    let mut tags =
        super::prepare_tag_list(tags.unwrap_or_else(|| vec![]).iter().map(String::as_str));
    if super::check_and_count_owned_tags(db, &tags, org.as_ref())? == 0 {
        if let Some(mut org) = org {
            // Implicitly add missing owned tags to prevent events with
            // undefined ownership!
            match mode {
                NewEventMode::Create => {
                    if created_by.is_none() {
                        // NOTE: At the moment we require an email address,
                        // but in the future we might allow anonymous creators
                        return Err(ParameterError::CreatorEmail.into());
                    }
                    if !org.owned_tags.is_empty() {
                        // Ensure that the newly created event is owned by the authorized org
                        log::info!(
                            "Implicitly adding all {} tag(s) owned by {} while creating event",
                            org.owned_tags.len(),
                            org.name
                        );
                        tags.reserve(org.owned_tags.len());
                        tags.append(&mut org.owned_tags);
                    }
                }
                NewEventMode::Update(id) => {
                    // Keep all existing tags owned by the authorized org
                    let old_tags = db.get_event(id)?.tags;
                    // Verify that the org is entitled to update this event according to the owned tags
                    let owned_count = super::check_and_count_owned_tags(db, &old_tags, Some(&org))?;
                    if owned_count > 0 {
                        debug_assert!(owned_count <= org.owned_tags.len());
                        tags.reserve(owned_count);
                        // Preserve all existing tags that are owned by this org
                        for owned_tag in old_tags
                            .into_iter()
                            .filter(|t| org.owned_tags.iter().any(|x| x == t))
                        {
                            tags.push(owned_tag);
                        }
                    }
                }
            }
        }
    }
    //TODO: use address.is_empty()
    let address = if street.is_some()
        || zip.is_some()
        || city.is_some()
        || country.is_some()
        || state.is_some()
    {
        Some(Address {
            street,
            zip,
            city,
            country,
            state,
        })
    } else {
        None
    };

    //TODO: use location.is_empty()
    let pos = if let (Some(lat), Some(lng)) = (lat, lng) {
        MapPoint::try_from_lat_lng_deg(lat, lng)
    } else {
        None
    };
    let location = if pos.is_some() || address.is_some() {
        Some(Location {
            pos: pos.unwrap_or_default(),
            address,
        })
    } else {
        None
    };
    //TODO: use contact.is_empty()
    let contact = if email.is_some() || telephone.is_some() {
        Some(Contact {
            email: email.map(Into::into),
            phone: telephone,
        })
    } else {
        None
    };

    let id = match mode {
        NewEventMode::Create => Id::new(),
        NewEventMode::Update(id) => Id::from(id),
    };

    let created_by = if let Some(ref email) = created_by {
        Some(create_user_from_email(db, email)?.email)
    } else {
        None
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
                            if c.phone.is_none() {
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

    let organizer = organizer
        .map(|x| x.trim().to_owned())
        .filter(|x| !x.is_empty());

    let start = NaiveDateTime::from_timestamp(start, 0);
    let end = end.map(|e| NaiveDateTime::from_timestamp(e, 0));

    let homepage = homepage
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image_url = image_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image_link_url = image_link_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;

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
        organizer,
        archived: None,
        image_url,
        image_link_url,
    };
    let event = event.auto_correct();
    event.validate()?;
    Ok(Storable(event))
}

pub fn store_created_event<D: Db>(db: &D, storable: Storable) -> Result<Event> {
    let Storable(event) = storable;
    debug!("Storing newly created event: {:?}", event);
    for t in &event.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_event(event.clone())?;
    Ok(event)
}

pub fn store_updated_event<D: Db>(db: &D, storable: Storable) -> Result<Event> {
    let Storable(event) = storable;
    debug!("Storing updated event: {:?}", event);
    for t in &event.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.update_event(&event)?;
    Ok(event)
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;

    fn create_new_event<D: Db>(db: &D, token: Option<&str>, e: NewEvent) -> Result<Event> {
        let s = import_new_event(db, token, e, NewEventMode::Create)?;
        store_created_event(db, s)
    }

    #[test]
    fn create_new_valid_event() {
        let now = Utc::now().naive_utc().timestamp();
        #[rustfmt::skip]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : now,
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            state        : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : Some(vec!["foo".into(),"bar".into()]),
            created_by   : Some("foo@bar.com".into()),
            registration : None,
            organizer    : None,
            image_url     : Some("http://somewhere.com/image_url.jpg".to_string()),
            image_link_url: Some("my.url/test.ext".to_string()),
        };
        let mock_db = MockDb::default();
        let id = create_new_event(&mock_db, None, x).unwrap().id;
        assert!(id.is_valid());
        assert_eq!(mock_db.events.borrow().len(), 1);
        assert_eq!(mock_db.tags.borrow().len(), 2);
        let x = &mock_db.events.borrow()[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.start.timestamp(), now);
        assert!(x.location.is_none());
        assert_eq!(x.description.as_ref().unwrap(), "bar");
        assert!(x.id.is_valid());
        assert_eq!(x.id, id);
        assert_eq!(
            "http://somewhere.com/image_url.jpg",
            x.image_url.as_ref().unwrap().as_str()
        );
        assert_eq!(
            "https://www.my.url/test.ext",
            x.image_link_url.as_ref().unwrap().as_str()
        );
    }

    #[test]
    fn create_event_with_invalid_email() {
        #[rustfmt::skip]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : Utc::now().naive_utc().timestamp(),
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            state        : None,
            email        : Some("fooo-not-ok".into()),
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : None,
            registration : None,
            organizer    : None,
            image_url     : None,
            image_link_url: None,
        };
        let mock_db: MockDb = MockDb::default();
        assert!(create_new_event(&mock_db, None, x).is_err());
    }

    #[test]
    fn create_event_with_valid_non_existing_creator_email() {
        #[rustfmt::skip]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : Utc::now().naive_utc().timestamp(),
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            state        : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : Some("fooo@bar.tld".into()),
            registration : None,
            organizer    : None,
            image_url     : None,
            image_link_url: None,
        };
        let mock_db: MockDb = MockDb::default();
        assert!(create_new_event(&mock_db, None, x).is_ok());
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(&users[0].email, "fooo@bar.tld");
    }

    #[test]
    fn create_event_with_valid_existing_creator_email() {
        let mock_db: MockDb = MockDb::default();
        mock_db
            .create_user(&User {
                email: "fooo@bar.tld".into(),
                email_confirmed: true,
                password: "secret".parse::<Password>().unwrap(),
                role: Role::User,
            })
            .unwrap();
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
        #[rustfmt::skip]
        let x = NewEvent {
            title        : "foo".into(),
            description  : Some("bar".into()),
            start        : Utc::now().naive_utc().timestamp(),
            end          : None,
            lat          : None,
            lng          : None,
            street       : None,
            zip          : None,
            city         : None,
            country      : None,
            state        : None,
            email        : None,
            telephone    : None,
            homepage     : None,
            tags         : None,
            created_by   : Some("fooo@bar.tld".into()),
            registration : None,
            organizer    : None,
            image_url     : None,
            image_link_url: None,
        };
        assert!(create_new_event(&mock_db, None, x).is_ok());
        let users = mock_db.all_users().unwrap();
        assert_eq!(users.len(), 1);
    }
}
