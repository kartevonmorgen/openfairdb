use std::collections::HashSet;

use chrono::NaiveDate;

use super::{parse_custom_link_param, CustomLinkParam};
use crate::core::{
    prelude::*,
    util::{parse::parse_url_param, validate::Validate},
};

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub struct NewPlace {
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
    pub founded_on     : Option<NaiveDate>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub license        : String,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
    pub custom_links   : Vec<CustomLinkParam>,
}

#[derive(Debug, Clone)]
pub struct Storable {
    place: Place,
    clearance_org_ids: Vec<Id>,
}

pub fn prepare_new_place<D: Db>(
    db: &D,
    e: NewPlace,
    created_by_email: Option<&str>,
    created_by_org: Option<&Organization>,
    accepted_licenses: &HashSet<String>,
) -> Result<Storable> {
    let NewPlace {
        title,
        description,
        categories,
        contact_name,
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
        license,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        custom_links: custom_links_param,
    } = e;
    let pos =
        MapPoint::try_from_lat_lng_deg(lat, lng).map_err(|_| ParameterError::InvalidPosition)?;

    let categories: Vec<_> = categories.into_iter().map(Id::from).collect();
    let old_tags = vec![];
    let new_tags = super::prepare_tag_list(
        Category::merge_ids_into_tags(&categories, tags)
            .iter()
            .map(String::as_str),
    );
    let clearance_org_ids =
        super::authorize_editing_of_tagged_entry(db, &old_tags, &new_tags, created_by_org)?;

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
    let location = Location { pos, address };

    let contact = if email.is_some() || telephone.is_some() {
        Some(Contact {
            name: contact_name,
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
        id: Id::new(),
        license,
        revision: Revision::initial(),
        created: Activity::now(created_by_email.map(Into::into)),
        title,
        description,
        location,
        contact,
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
    })
}

pub fn store_new_place<D: Db>(db: &D, s: Storable) -> Result<(Place, Vec<Rating>)> {
    let Storable {
        place,
        clearance_org_ids,
    } = s;
    debug!("Storing new place revision: {:?}", place);
    for t in &place.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_or_update_place(place.clone())?;
    if !clearance_org_ids.is_empty() {
        let pending_clearance = PendingClearanceForPlace {
            place_id: place.id.clone(),
            created_at: place.created.at,
            last_cleared_revision: None,
        };
        super::clearance::place::add_pending_clearance(db, &clearance_org_ids, &pending_clearance)?;
    }
    // No initial ratings so far
    let ratings = vec![];
    Ok((place, ratings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::usecases::tests::MockDb, infrastructure::cfg::Cfg};

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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec![],
            license     : "ODbL-1.0".into(),
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db = MockDb::default();
        let now = TimestampMs::now();
        let storable = prepare_new_place(
            &mock_db,
            x,
            Some("test@example.com"),
            None,
            &Cfg::default().accepted_licenses,
        )
        .unwrap();
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
            state       : None,
            contact_name: None,
            email       : Some("fooo-not-ok".into()),
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec![],
            license     : "ODbL-1.0".into(),
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db: MockDb = MockDb::default();
        assert!(
            prepare_new_place(&mock_db, x, None, None, &Cfg::default().accepted_licenses).is_err()
        );
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
            state       : None,
            contact_name: None,
            email       : None,
            telephone   : None,
            homepage    : None,
            opening_hours: None,
            founded_on  : None,
            categories  : vec![],
            tags        : vec!["foo".into(),"bar".into()],
            license     : "ODbL-1.0".into(),
            image_url     : None,
            image_link_url: None,
            custom_links: vec![],
        };
        let mock_db = MockDb::default();
        let e =
            prepare_new_place(&mock_db, x, None, None, &Cfg::default().accepted_licenses).unwrap();
        assert!(store_new_place(&mock_db, e).is_ok());
        assert_eq!(mock_db.tags.borrow().len(), 2);
        assert_eq!(mock_db.entries.borrow().len(), 1);
    }
}
