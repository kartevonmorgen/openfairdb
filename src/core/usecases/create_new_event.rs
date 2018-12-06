use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};
use uuid::Uuid;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Deserialize, Debug, Clone)]
pub struct NewEvent {
    pub title          : String,
    pub description    : Option<String>,
    pub start          : u64,
    pub end            : Option<u64>,
    pub lat            : Option<f64>,
    pub lng            : Option<f64>,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub tags           : Option<Vec<String>>,
}

pub fn create_new_event<D: Db>(db: &mut D, e: NewEvent) -> Result<String> {
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
        ..
    } = e;
    let mut tags: Vec<_> = tags
        .unwrap_or_else(|| vec![])
        .into_iter()
        .map(|t| t.replace("#", ""))
        .collect();
    tags.dedup();
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
    let homepage = e.homepage.map(|ref url| parse_url_param(url)).transpose()?;

    let new_event = Event {
        id,
        title,
        start,
        end,
        description,
        location,
        contact,
        homepage,
        tags,
    };

    debug!("Creating new event: {:?}", new_event);
    new_event.validate()?;
    for t in &new_event.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_event(&new_event)?;
    Ok(new_event.id)
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
            title       : "foo".into(),
            description : Some("bar".into()),
            start       : 9999,
            end         : None,
            lat         : None,
            lng         : None,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            tags        : Some(vec!["foo".into(),"bar".into()]),
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
            title       : "foo".into(),
            description : Some("bar".into()),
            start       : 9999,
            end         : None,
            lat         : None,
            lng         : None,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : Some("fooo-not-ok".into()),
            telephone   : None,
            homepage    : None,
            tags        : None,
        };
        let mut mock_db: MockDb = MockDb::new();
        assert!(create_new_event(&mut mock_db, x).is_err());
    }
}
