use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone)]
pub struct NewPlace {
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
pub struct Storable(Place);

pub fn prepare_new_place<D: Db>(
    db: &D,
    e: NewPlace,
    created_by_email: Option<&str>,
) -> Result<Storable> {
    let NewPlace {
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
        license,
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
            email: email.map(Into::into),
            phone: telephone,
        })
    } else {
        None
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
        id: Id::new(),
        license,
        revision: Revision::initial(),
        created: Activity::now(created_by_email.map(Into::into)),
        title,
        description,
        location,
        contact,
        links,
        tags,
    };
    place.validate()?;
    Ok(Storable(place))
}

pub fn store_new_place<D: Db>(db: &D, s: Storable) -> Result<(Place, Vec<Rating>)> {
    let Storable(place) = s;
    debug!("Storing new place revision: {:?}", place);
    for t in &place.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_or_update_place(place.clone())?;
    // No initial ratings so far
    let ratings = vec![];
    Ok((place, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;

    #[test]
    fn create_new_valid_place() {
        #[rustfmt::skip]
        let x = NewPlace {
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
        let now = TimestampMs::now();
        let storable = prepare_new_place(&mock_db, x, Some("test@example.com")).unwrap();
        let (_, initial_ratings) = store_new_place(&mock_db, storable).unwrap();
        assert!(initial_ratings.is_empty());
        assert_eq!(mock_db.entries.borrow().len(), 1);
        let (x, _) = &mock_db.entries.borrow()[0];
        assert_eq!(x.title, "foo");
        assert_eq!(x.description, "bar");
        assert!(x.created.at >= now);
        assert_eq!(x.created.by, Some("test@example.com".into()));
        assert_eq!(x.revision, Revision::initial());
    }

    #[test]
    fn create_place_with_invalid_email() {
        #[rustfmt::skip]
        let x = NewPlace {
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
        assert!(prepare_new_place(&mock_db, x, None).is_err());
    }

    #[test]
    fn add_new_valid_place_with_tags() {
        #[rustfmt::skip]
        let x = NewPlace {
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
        let e = prepare_new_place(&mock_db, x, None).unwrap();
        assert!(store_new_place(&mock_db, e).is_ok());
        assert_eq!(mock_db.tags.borrow().len(), 2);
        assert_eq!(mock_db.entries.borrow().len(), 1);
    }
}
