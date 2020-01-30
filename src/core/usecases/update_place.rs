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

pub fn prepare_updated_place<D: Db>(
    db: &D,
    place_id: Id,
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
        homepage,
        image_url,
        image_link_url,
        ..
    } = e;
    let pos = match MapPoint::try_from_lat_lng_deg(lat, lng) {
        None => return Err(ParameterError::InvalidPosition.into()),
        Some(pos) => pos,
    };
    let categories: Vec<_> = categories.into_iter().map(Id::from).collect();
    let tags = super::prepare_tag_list(
        Category::merge_ids_into_tags(&categories, tags)
            .iter()
            .map(String::as_str),
    );
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
        let (old_place, _) = db.get_place(place_id.as_str())?;
        // Check for revision conflict (optimistic locking)
        let revision = Revision::from(version);
        if old_place.revision.next() != revision {
            return Err(RepoError::InvalidVersion.into());
        }
        // The license is immutable
        let license = old_place.license;
        (revision, license)
    };

    let homepage = homepage
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image = image_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image_href = image_link_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let links = if homepage.is_some() || image.is_some() || image_href.is_some() {
        Some(Links {
            homepage,
            image,
            image_href,
        })
    } else {
        None
    };

    let place = Place {
        id: place_id,
        license,
        revision,
        created: Activity::now(updated_by.map(Into::into)),
        title,
        description,
        location: Location { pos, address },
        contact: Some(Contact {
            email: email.map(Into::into),
            phone,
        }),
        links,
        tags,
    };
    place.validate()?;
    Ok(Storable(place))
}

pub fn store_updated_place<D: Db>(db: &D, s: Storable) -> Result<(Place, Vec<Rating>)> {
    let Storable(place) = s;
    debug!("Storing updated place revision: {:?}", place);
    for t in &place.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_or_update_place(place.clone())?;
    let ratings = db.load_ratings_of_place(place.id.as_ref())?;
    Ok((place, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;

    use url::Url;

    #[test]
    fn update_place_valid() {
        let id = Id::new();
        let old = Place::build()
            .id(id.as_ref())
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
            image_link_url: old.links.as_ref().and_then(|l| l.image_href.as_ref()).map(|url| url.as_str().to_string()),
        };
        let mut mock_db = MockDb::default();
        mock_db.entries = vec![(old, ReviewStatus::Created)].into();
        let now = TimestampMs::now();
        let storable = prepare_updated_place(&mock_db, id, new, Some("test@example.com")).unwrap();
        assert!(store_updated_place(&mock_db, storable).is_ok());
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
        assert_eq!(Revision::from(2), x.revision);
        assert!(x.created.at >= now);
        assert_eq!(
            Some("test@example.com"),
            x.created.by.as_ref().map(Email::as_ref)
        );
        assert_eq!(
            Some("https://www.img2/"),
            x.links
                .as_ref()
                .and_then(|l| l.image.as_ref())
                .map(Url::as_str)
        );
        assert_eq!(
            Some("http://imglink/"),
            x.links
                .as_ref()
                .and_then(|l| l.image_href.as_ref())
                .map(Url::as_str)
        );
    }

    #[test]
    fn update_place_with_invalid_version() {
        let id = Id::new();
        let old = Place::build()
            .id(id.as_ref())
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
        mock_db.entries = vec![(old, ReviewStatus::Created)].into();
        let err = match prepare_updated_place(&mock_db, id, new, None) {
            Ok(storable) => store_updated_place(&mock_db, storable).err(),
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
    fn update_non_existing_place() {
        let id = Id::new();
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
        let result = prepare_updated_place(&mock_db, id, new, None);
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
    fn update_place_with_tags() {
        let id = Id::new();
        let old = Place::build()
            .id(id.as_ref())
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
        mock_db.entries = vec![(old, ReviewStatus::Created)].into();
        mock_db.tags = vec![Tag { id: "bio".into() }, Tag { id: "fair".into() }].into();
        let storable = prepare_updated_place(&mock_db, id.clone(), new, None).unwrap();
        assert!(store_updated_place(&mock_db, storable).is_ok());
        let (e, _) = mock_db.get_place(id.as_ref()).unwrap();
        assert_eq!(e.tags, vec!["vegan"]);
        assert_eq!(mock_db.tags.borrow().len(), 3);
    }
}
