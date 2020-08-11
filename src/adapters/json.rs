use crate::core::{db::IndexedPlace, entities as e, usecases};
use url::Url;

pub use ofdb_boundary::*;

impl From<IndexedPlace> for PlaceSearchResult {
    fn from(from: IndexedPlace) -> Self {
        let IndexedPlace {
            id,
            status,
            title,
            description,
            tags,
            pos,
            ratings,
            ..
        } = from;
        // The status should never be undefined! It is optional only
        // for technical reasons.
        debug_assert!(status.is_some());
        let status = status.map(Into::into);
        let (tags, categories) = e::Category::split_from_tags(tags);
        let categories = categories.into_iter().map(|c| c.id.to_string()).collect();
        let lat = pos.lat().to_deg();
        let lng = pos.lng().to_deg();
        let e::AvgRatings {
            diversity,
            fairness,
            humanity,
            renewable,
            solidarity,
            transparency,
        } = ratings;
        let total = ratings.total().into();
        let ratings = EntrySearchRatings {
            total,
            diversity: diversity.into(),
            fairness: fairness.into(),
            humanity: humanity.into(),
            renewable: renewable.into(),
            solidarity: solidarity.into(),
            transparency: transparency.into(),
        };
        Self {
            id,
            status,
            lat,
            lng,
            title,
            description,
            categories,
            tags,
            ratings,
        }
    }
}

impl From<NewPlace> for usecases::NewPlace {
    fn from(p: NewPlace) -> Self {
        let NewPlace {
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            email,
            telephone,
            homepage,
            opening_hours,
            categories,
            tags,
            license,
            image_url,
            image_link_url,
        } = p;
        usecases::NewPlace {
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            email,
            telephone,
            homepage,
            opening_hours,
            categories,
            tags,
            license,
            image_url,
            image_link_url,
        }
    }
}

impl From<UpdatePlace> for usecases::UpdatePlace {
    fn from(p: UpdatePlace) -> Self {
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
            email,
            telephone,
            homepage,
            opening_hours,
            categories,
            tags,
            image_url,
            image_link_url,
        } = p;
        usecases::UpdatePlace {
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
            email,
            telephone,
            homepage,
            opening_hours,
            categories,
            tags,
            image_url,
            image_link_url,
        }
    }
}

// Entity -> JSON

pub fn entry_from_place_with_ratings(place: e::Place, ratings: Vec<e::Rating>) -> Entry {
    let e::Place {
        id,
        license,
        revision,
        created,
        title,
        description,
        location,
        contact,
        opening_hours,
        links,
        tags,
    } = place;

    let e::Location { pos, address } = location;
    let lat = pos.lat().to_deg();
    let lng = pos.lng().to_deg();
    let e::Address {
        street,
        zip,
        city,
        country,
        state,
    } = address.unwrap_or_default();

    let e::Contact {
        email,
        phone: telephone,
    } = contact.unwrap_or_default();

    let (homepage_url, image_url, image_link_url) = if let Some(links) = links {
        (links.homepage, links.image, links.image_href)
    } else {
        (None, None, None)
    };

    let (tags, categories) = e::Category::split_from_tags(tags);

    Entry {
        id: id.into(),
        created: created.at.into_seconds(),
        version: revision.into(),
        title,
        description,
        lat,
        lng,
        street,
        zip,
        city,
        country,
        state,
        email: email.map(Into::into),
        telephone,
        homepage: homepage_url.map(Url::into_string),
        opening_hours: opening_hours.map(Into::into),
        categories: categories.into_iter().map(|c| c.id.to_string()).collect(),
        tags,
        ratings: ratings.into_iter().map(|r| r.id.to_string()).collect(),
        license: Some(license),
        image_url: image_url.map(Url::into_string),
        image_link_url: image_link_url.map(Url::into_string),
    }
}

#[derive(Debug, Deserialize)]
pub struct Review {
    pub status: ReviewStatus,
    pub comment: Option<String>,
}

