use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};

#[rustfmt::skip]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePlace {
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

pub struct Storable(Place);

pub fn prepare_updated_place_rev<D: Db>(
    db: &D,
    place_uid: Uid,
    e: UpdatePlace,
    updated_by: Option<&str>,
) -> Result<Storable> {
    let UpdatePlace {
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
        telephone: phone,
        categories,
        tags,
        ..
    } = e;
    let pos = match MapPoint::try_from_lat_lng_deg(lat, lng) {
        None => return Err(ParameterError::InvalidPosition.into()),
        Some(pos) => pos,
    };
    let categories = categories.into_iter().map(Uid::from).collect();
    let tags = super::prepare_tag_list(Category::merge_uids_into_tags(categories, tags));
    super::check_and_count_owned_tags(db, &tags, None)?;
    // TODO: Ensure that no reserved tags are removed without authorization.
    // All existing reserved tags from other organizations must be preserved
    // when editing places. Reserved tags that already exist should not be
    // considers during the check, because they must be preserved independent
    // of who is editing the place_rev.
    // GitHub issue: https://github.com/slowtec/openfairdb/issues/203
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
    let (revision, license) = {
        let old_place_rev = db.get_place(place_uid.as_str())?.0;
        // Check for revision conflict (optimistic locking)
        let revision = Revision::from(version);
        if old_place_rev.rev.next() != revision {
            return Err(RepoError::InvalidVersion.into());
        }
        // The license is immutable
        let license = old_place_rev.license;
        (revision, license)
    };
    let plave_rev = Place {
        uid: place_uid,
        rev: revision,
        created: Activity::now(updated_by.map(Into::into)),
        license,
        title,
        description,
        location: Location { pos, address },
        contact: Some(Contact {
            email: email.map(Into::into),
            phone,
        }),
        homepage: e.homepage.map(|ref url| parse_url_param(url)).transpose()?,
        image_url: e
            .image_url
            .map(|ref url| parse_url_param(url))
            .transpose()?,
        image_link_url: e
            .image_link_url
            .map(|ref url| parse_url_param(url))
            .transpose()?,
        tags,
    };
    plave_rev.validate()?;
    Ok(Storable(plave_rev))
}

pub fn store_updated_place_rev<D: Db>(db: &D, s: Storable) -> Result<(Place, Vec<Rating>)> {
    let Storable(place_rev) = s;
    debug!("Storing updated place revision: {:?}", place_rev);
    for t in &place_rev.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_place_rev(place_rev.clone())?;
    let ratings = db.load_ratings_of_entry(place_rev.uid.as_ref())?;
    Ok((place_rev, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;

    #[test]
    fn update_valid_place_rev() {
        let uid = Uid::new_uuid();
        let old = Place::build()
            .id(uid.as_ref())
            .revision(1)
            .title("foo")
            .description("bar")
            .image_url(Some("http://img"))
            .image_link_url(Some("http://imglink"))
            .license("CC0-1.0")
            .finish();

        #[rustfmt::skip]
        let new = UpdatePlace {
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
        mock_db.entries = vec![(old, Status::created())].into();
        let now = Timestamp::now();
        let storable =
            prepare_updated_place_rev(&mock_db, uid.clone(), new, Some("test@example.com"))
                .unwrap();
        assert!(store_updated_place_rev(&mock_db, storable).is_ok());
        assert_eq!(mock_db.entries.borrow().len(), 1);
        let (x, _) = &mock_db.entries.borrow()[0];
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
        assert_eq!(Revision::from(2), x.rev);
        assert!(x.created.when >= now);
        assert_eq!(x.created.who, Some("test@example.com".into()));
        assert_eq!("https://www.img2/", x.image_url.as_ref().unwrap());
        assert_eq!("http://imglink/", x.image_link_url.as_ref().unwrap());
    }

    #[test]
    fn update_place_rev_with_invalid_version() {
        let uid = Uid::new_uuid();
        let old = Place::build()
            .id(uid.as_ref())
            .revision(3)
            .title("foo")
            .description("bar")
            .license("CC0-1.0")
            .finish();

        #[rustfmt::skip]
        let new = UpdatePlace {
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
        mock_db.entries = vec![(old, Status::created())].into();
        let err = match prepare_updated_place_rev(&mock_db, uid.clone(), new, None) {
            Ok(storable) => store_updated_place_rev(&mock_db, storable).err(),
            Err(err) => Some(err),
        };
        assert!(err.is_some());
        match err.unwrap() {
            Error::Repo(err) => match err {
                RepoError::InvalidVersion => {}
                e => {
                    panic!(format!("Unexpected error: {:?}", e));
                }
            },
            e => {
                panic!(format!("Unexpected error: {:?}", e));
            }
        }
        assert_eq!(mock_db.entries.borrow().len(), 1);
    }

    #[test]
    fn update_non_existing_place_rev() {
        let uid = Uid::new_uuid();
        #[rustfmt::skip]
        let new = UpdatePlace {
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
        let result = prepare_updated_place_rev(&mock_db, uid.clone(), new, None);
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
    fn update_valid_place_rev_with_tags() {
        let uid = Uid::new_uuid();
        let old = Place::build()
            .id(uid.as_ref())
            .revision(1)
            .tags(vec!["bio", "fair"])
            .license("CC0-1.0")
            .finish();
        #[rustfmt::skip]
        let new = UpdatePlace {
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
        mock_db.entries = vec![(old, Status::created())].into();
        mock_db.tags = vec![Tag { id: "bio".into() }, Tag { id: "fair".into() }].into();
        let storable = prepare_updated_place_rev(&mock_db, uid.clone(), new, None).unwrap();
        assert!(store_updated_place_rev(&mock_db, storable).is_ok());
        let (e, _) = mock_db.get_place(uid.as_ref()).unwrap();
        assert_eq!(e.tags, vec!["vegan"]);
        assert_eq!(mock_db.tags.borrow().len(), 3);
    }
}
