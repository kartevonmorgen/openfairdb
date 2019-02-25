use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};
use chrono::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    pub osm_node       : Option<u64>,
    pub version        : u64,
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
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
}

pub struct Storable(Entry);

pub fn prepare_updated_entry<D: Db>(db: &D, id: String, e: UpdateEntry) -> Result<Storable> {
    let old: Entry = db.get_entry(&id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion));
    }
    let UpdateEntry {
        version,
        title,
        description,
        lat,
        lng,
        street,
        zip,
        city,
        country,
        email,
        telephone,
        categories,
        tags,
        ..
    } = e;
    let tags = super::prepare_tag_list(tags);
    super::check_for_owned_tags(db, &tags, &None)?;
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
    let e = Entry {
        id,
        osm_node: None,
        created: Utc::now().timestamp() as u64,
        version,
        title,
        description,
        location: Location { lat, lng, address },
        contact: Some(Contact { email, telephone }),
        homepage: e.homepage.map(|ref url| parse_url_param(url)).transpose()?,
        categories,
        tags,
        license: old.license, // license is immutable
        image_url: e
            .image_url
            .map(|ref url| parse_url_param(url))
            .transpose()?,
        image_link_url: e
            .image_link_url
            .map(|ref url| parse_url_param(url))
            .transpose()?,
    };
    e.validate()?;
    Ok(Storable(e))
}

pub fn store_updated_entry<D: Db>(db: &D, s: Storable) -> Result<(Entry, Vec<Rating>)> {
    let Storable(entry) = s;
    debug!("Storing updated entry: {:?}", entry);
    for t in &entry.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.update_entry(&entry)?;
    let ratings = db.all_ratings_for_entry_by_id(&entry.id)?;
    Ok((entry, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
    use uuid::Uuid;

    #[test]
    fn update_valid_entry() {
        let id = Uuid::new_v4().to_simple_ref().to_string();
        let old = Entry::build()
            .id(&id)
            .version(1)
            .title("foo")
            .description("bar")
            .image_url(Some("http://img"))
            .image_link_url(Some("http://imglink"))
            .license(Some("CC0-1.0"))
            .finish();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            osm_node    :  None,
            version     : 2,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : Some("img2".into()),
            image_link_url: old.image_link_url.clone(),
        };
        let mut mock_db = MockDb::default();
        mock_db.entries = vec![old].into();
        let now = Utc::now();
        let e = prepare_updated_entry(&mock_db, id.clone(), new).unwrap();
        assert!(store_updated_entry(&mock_db, e).is_ok());
        assert_eq!(mock_db.entries.borrow().len(), 1);
        let x = &mock_db.entries.borrow()[0];
        assert_eq!(
            "street",
            x.location
                .address
                .as_ref()
                .unwrap()
                .street
                .as_ref()
                .unwrap()
        );
        assert_eq!("bar", x.description);
        assert_eq!(2, x.version);
        assert!(x.created as i64 >= now.timestamp());
        assert!(Uuid::parse_str(&x.id).is_ok());
        assert_eq!("https://www.img2/", x.image_url.as_ref().unwrap());
        assert_eq!("http://imglink/", x.image_link_url.as_ref().unwrap());
    }

    #[test]
    fn update_entry_with_invalid_version() {
        let id = Uuid::new_v4().to_simple_ref().to_string();
        let old = Entry::build()
            .id(&id)
            .version(3)
            .title("foo")
            .description("bar")
            .finish();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            osm_node    : None,
            version     : 3,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : None,
            image_link_url: None,
        };
        let mut mock_db = MockDb::default();
        mock_db.entries = vec![old].into();
        let result = prepare_updated_entry(&mock_db, id.clone(), new);
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::Repo(err) => match err {
                RepoError::InvalidVersion => {}
                _ => {
                    panic!("invalid error type");
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.entries.borrow().len(), 1);
    }

    #[test]
    fn update_non_existing_entry() {
        let id = Uuid::new_v4().to_simple_ref().to_string();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            osm_node    : None,
            version     : 4,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : None,
            image_link_url: None,
        };
        let mut mock_db = MockDb::default();
        mock_db.entries = vec![].into();
        let result = prepare_updated_entry(&mock_db, id.clone(), new);
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::Repo(err) => match err {
                RepoError::NotFound => {}
                _ => {
                    panic!("invalid error type");
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.entries.borrow().len(), 0);
    }

    #[test]
    fn update_valid_entry_with_tags() {
        let id = Uuid::new_v4().to_simple_ref().to_string();
        let old = Entry::build()
            .id(&id)
            .version(1)
            .tags(vec!["bio", "fair"])
            .license(Some("CC0-1.0"))
            .finish();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            osm_node    :  None,
            version     : 2,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : Some("street".into()),
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec!["vegan".into()],
            image_url     : None,
            image_link_url: None,
        };
        let mut mock_db = MockDb::default();
        mock_db.entries = vec![old].into();
        mock_db.tags = vec![Tag { id: "bio".into() }, Tag { id: "fair".into() }].into();
        let e = prepare_updated_entry(&mock_db, id.clone(), new).unwrap();
        assert!(store_updated_entry(&mock_db, e).is_ok());
        let e = mock_db.get_entry(&id).unwrap();
        assert_eq!(e.tags, vec!["vegan"]);
        assert_eq!(mock_db.tags.borrow().len(), 3);
    }

}
