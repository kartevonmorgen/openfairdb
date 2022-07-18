use std::collections::HashSet;

use time::Date;

use super::{parse_custom_link_param, CustomLinkParam};
use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};

#[rustfmt::skip]
#[derive(Debug, Clone)]
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
    pub state          : Option<String>,
    pub contact_name   : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub opening_hours  : Option<String>,
    pub founded_on     : Option<Date>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
    pub custom_links   : Vec<CustomLinkParam>,
}

impl From<Place> for UpdatePlace {
    fn from(from: Place) -> Self {
        let Place {
            contact,
            created: _,
            description,
            id: _,
            license: _,
            links,
            location: Location { address, pos },
            opening_hours,
            founded_on,
            revision,
            tags,
            title,
        } = from;
        let (city, country, state, street, zip) = address
            .map(|a| (a.city, a.country, a.state, a.street, a.zip))
            .unwrap_or_default();
        let (homepage_url, image_url, image_link_url, custom_links) = links
            .map(
                |Links {
                     homepage,
                     image,
                     image_href,
                     custom,
                 }| (homepage, image, image_href, custom),
            )
            .unwrap_or_default();
        let (contact_name, email, telephone) = contact
            .map(|c| (c.name, c.email, c.phone))
            .unwrap_or_default();
        Self {
            categories: vec![],
            city,
            country,
            custom_links: custom_links.into_iter().map(Into::into).collect(),
            description,
            contact_name,
            email: email.map(Into::into),
            homepage: homepage_url.map(|url| url.to_string()),
            image_link_url: image_link_url.map(|url| url.to_string()),
            image_url: image_url.map(|url| url.to_string()),
            lat: pos.lat().to_deg(),
            lng: pos.lng().to_deg(),
            opening_hours: opening_hours.map(Into::into),
            founded_on,
            state,
            street,
            tags,
            telephone,
            title,
            version: revision.into(),
            zip,
        }
    }
}

pub struct Storable {
    place: Place,
    clearance_org_ids: Vec<Id>,
    last_cleared_revision: Revision,
}

pub fn prepare_updated_place<D: Db>(
    db: &D,
    place_id: Id,
    e: UpdatePlace,
    created_by_email: Option<&str>,
    created_by_org: Option<&Organization>,
    accepted_licenses: &HashSet<String>,
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
        state,
        contact_name,
        email,
        telephone: phone,
        opening_hours,
        founded_on,
        categories,
        tags,
        homepage,
        image_url,
        image_link_url,
        custom_links: custom_links_param,
        ..
    } = e;
    let pos =
        MapPoint::try_from_lat_lng_deg(lat, lng).map_err(|_| ParameterError::InvalidPosition)?;
    let address = Address {
        street,
        zip,
        city,
        country,
        state,
    };
    let address = if address.is_empty() {
        None
    } else {
        Some(address)
    };

    let (revision, last_cleared_revision, old_tags, license) = {
        let (old_place, _review_status) = db.get_place(place_id.as_str())?;
        // Check for revision conflict (optimistic locking)
        let revision = Revision::from(version);
        if old_place.revision.next() != revision {
            return Err(RepoError::InvalidVersion.into());
        }
        let last_cleared_revision = old_place.revision;
        // The license is immutable
        let license = old_place.license;
        // The existing tags are needed for authorization
        let old_tags = old_place.tags;
        (revision, last_cleared_revision, old_tags, license)
    };

    let categories: Vec<_> = categories.into_iter().map(Id::from).collect();
    let new_tags = super::prepare_tag_list(
        Category::merge_ids_into_tags(&categories, tags)
            .iter()
            .map(String::as_str),
    );
    let clearance_org_ids =
        super::authorize_editing_of_tagged_entry(db, &old_tags, &new_tags, created_by_org)?;

    let homepage = homepage
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image = image_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let image_href = image_link_url
        .and_then(|ref url| parse_url_param(url).transpose())
        .transpose()?;
    let mut custom_links = Vec::with_capacity(custom_links_param.len());
    for custom_link_param in custom_links_param {
        custom_links.push(parse_custom_link_param(custom_link_param)?);
    }
    let links =
        if homepage.is_none() && image.is_none() && image_href.is_none() && custom_links.is_empty()
        {
            None
        } else {
            Some(Links {
                homepage,
                image,
                image_href,
                custom: custom_links,
            })
        };

    let place = Place {
        id: place_id,
        license,
        revision,
        created: Activity::now(created_by_email.map(Into::into)),
        title,
        description,
        location: Location { pos, address },
        contact: Some(Contact {
            name: contact_name,
            email: email.map(Into::into),
            phone,
        }),
        opening_hours: opening_hours
            .map(|s| {
                s.parse()
                    .map_err(|_| Error::Parameter(ParameterError::InvalidOpeningHours))
            })
            .transpose()?,
        founded_on,
        links,
        tags: new_tags,
    };
    place.validate()?;
    if !accepted_licenses.contains(&place.license) {
        return Err(Error::Parameter(ParameterError::License));
    }
    Ok(Storable {
        place,
        clearance_org_ids,
        last_cleared_revision,
    })
}

pub fn store_updated_place<D: Db>(db: &D, s: Storable) -> Result<(Place, Vec<Rating>)> {
    let Storable {
        place,
        clearance_org_ids,
        last_cleared_revision,
    } = s;
    debug!("Storing updated place revision: {:?}", place);
    for t in &place.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_or_update_place(place.clone())?;
    if !clearance_org_ids.is_empty() {
        let pending_clearance = PendingClearanceForPlace {
            place_id: place.id.clone(),
            created_at: place.created.at,
            last_cleared_revision: Some(last_cleared_revision),
        };
        super::clearance::place::add_pending_clearance(db, &clearance_org_ids, &pending_clearance)?;
    }
    let ratings = db.load_ratings_of_place(place.id.as_ref())?;
    Ok((place, ratings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::usecases::tests::MockDb, infrastructure::cfg::Cfg};

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
            .license("ODbL-1.0")
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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : Some("img2".into()),
            image_link_url: old.links.as_ref().and_then(|l| l.image_href.as_ref()).map(|url| url.as_str().to_string()),
            custom_links: vec![],
        };
        let mock_db = MockDb {
            entries: vec![(old, ReviewStatus::Created)].into(),
            ..Default::default()
        };
        let now = Timestamp::now();
        let storable = prepare_updated_place(
            &mock_db,
            id,
            new,
            Some("test@example.com"),
            None,
            &Cfg::default().accepted_licenses,
        )
        .unwrap();
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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db = MockDb {
            entries: vec![(old, ReviewStatus::Created)].into(),
            ..Default::default()
        };
        let err = match prepare_updated_place(
            &mock_db,
            id,
            new,
            None,
            None,
            &Cfg::default().accepted_licenses,
        ) {
            Ok(storable) => store_updated_place(&mock_db, storable).err(),
            Err(err) => Some(err),
        };
        assert!(err.is_some());
        match err.unwrap() {
            Error::Repo(RepoError::InvalidVersion) => {
                // ok
            }
            e => {
                panic!("Unexpected error: {:?}", e);
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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec![],
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db = MockDb::default();
        let result = prepare_updated_place(
            &mock_db,
            id,
            new,
            None,
            None,
            &Cfg::default().accepted_licenses,
        );
        assert!(result.is_err());
        match result.err().unwrap() {
            Error::Repo(RepoError::NotFound) => {
                // ok
            }
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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec!["vegan".into()],
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db = MockDb {
            entries: vec![(old, ReviewStatus::Created)].into(),
            tags: vec![Tag { id: "bio".into() }, Tag { id: "fair".into() }].into(),
            ..Default::default()
        };
        let storable = prepare_updated_place(
            &mock_db,
            id.clone(),
            new,
            None,
            None,
            &Cfg::default().accepted_licenses,
        )
        .unwrap();
        assert!(store_updated_place(&mock_db, storable).is_ok());
        let (e, _) = mock_db.get_place(id.as_ref()).unwrap();
        assert_eq!(e.tags, vec!["vegan"]);
        assert_eq!(mock_db.tags.borrow().len(), 3);
    }
}
