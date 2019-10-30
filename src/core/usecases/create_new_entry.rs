use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone)]
pub struct NewEntry {
    pub title          : String,
    pub description    : String,
    pub lat            : f64,
    pub lng            : f64,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub license        : String,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
}

#[derive(Debug, Clone)]
pub struct Storable(Entry);

pub fn prepare_new_entry<D: Db>(db: &D, e: NewEntry) -> Result<Storable> {
    let NewEntry {
        title,
        description,
        categories,
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
    let pos = match MapPoint::try_from_lat_lng_deg(lat, lng) {
        None => return Err(ParameterError::InvalidPosition.into()),
        Some(pos) => pos,
    };
    let tags = super::prepare_tag_list(tags);
    super::check_and_count_owned_tags(db, &tags, None)?;
    let address = Address {
        street,
        zip,
        city,
        country,
    };
    let address = if address.is_empty() {
        None
    } else {
        Some(address)
    };
    let location = Location { pos, address };
    let contact = if email.is_some() || telephone.is_some() {
        Some(Contact {
            email,
            phone: telephone,
        })
    } else {
        None
    };
    let created_at = Timestamp::now();
    let new_id = Uid::new_uuid();
    let uid = new_id.clone();
    let homepage = e.homepage.map(|ref url| parse_url_param(url)).transpose()?;
    let image_url = e
        .image_url
        .map(|ref url| parse_url_param(url))
        .transpose()?;
    let image_link_url = e
        .image_link_url
        .map(|ref url| parse_url_param(url))
        .transpose()?;

    let e = Entry {
        osm_node: None,
        uid,
        created_at,
        archived_at: None,
        version: 0,
        title,
        description,
        location,
        contact,
        homepage,
        categories,
        tags,
        license: Some(e.license),
        image_url,
        image_link_url,
    };
    e.validate()?;
    Ok(Storable(e))
}

pub fn store_new_entry<D: Db>(db: &D, s: Storable) -> Result<(Entry, Vec<Rating>)> {
    let Storable(entry) = s;
    debug!("Storing newly created entry: {:?}", entry);
    for t in &entry.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_entry(entry.clone())?;
    // No initial ratings so far
    let ratings = vec![];
    Ok((entry, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;

    #[test]
    fn create_new_valid_entry() {
        #[rustfmt::skip]
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into(),
            image_url     : None,
            image_link_url: None,
        };
        let mock_db = MockDb::default();
        let now = Timestamp::now();
        let e = prepare_new_entry(&mock_db, x).unwrap();
        let (e, initial_ratings) = store_new_entry(&mock_db, e).unwrap();
        assert!(initial_ratings.is_empty());
        assert_eq!(&e.uid, &e.uid.as_ref().parse().unwrap());
        assert_eq!(mock_db.entries.borrow().len(), 1);
        let x = &mock_db.entries.borrow()[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.description, "bar");
        assert_eq!(x.version, 0);
        assert!(x.created_at >= now);
        assert_eq!(None, x.archived_at);
        assert_eq!(&x.uid, &x.uid.as_ref().parse().unwrap());
        assert_eq!(x.uid, e.uid);
    }

    #[test]
    fn create_entry_with_invalid_email() {
        #[rustfmt::skip]
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : Some("fooo-not-ok".into()),
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : "CC0-1.0".into(),
            image_url     : None,
            image_link_url: None,
        };
        let mock_db: MockDb = MockDb::default();
        assert!(prepare_new_entry(&mock_db, x).is_err());
    }

    #[test]
    fn add_new_valid_entry_with_tags() {
        #[rustfmt::skip]
        let x = NewEntry {
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec!["foo".into(),"bar".into()],
            license     : "CC0-1.0".into(),
            image_url     : None,
            image_link_url: None,
        };
        let mock_db = MockDb::default();
        let e = prepare_new_entry(&mock_db, x).unwrap();
        assert!(store_new_entry(&mock_db, e).is_ok());
        assert_eq!(mock_db.tags.borrow().len(), 2);
        assert_eq!(mock_db.entries.borrow().len(), 1);
    }
}
