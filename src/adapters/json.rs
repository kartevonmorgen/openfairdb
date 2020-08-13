use crate::core::{db::IndexedPlace, entities as e, usecases};

use url::Url;

pub use ofdb_boundary::*;

impl From<Credentials> for usecases::Login {
    fn from(from: Credentials) -> Self {
        let Credentials { email, password } = from;
        Self { email, password }
    }
}

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

impl From<CustomLink> for usecases::CustomLinkParam {
    fn from(from: CustomLink) -> Self {
        let CustomLink {
            url,
            title,
            description,
        } = from;
        Self {
            url,
            title,
            description,
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
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            license,
            image_url,
            image_link_url,
            links,
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
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            license,
            image_url,
            image_link_url,
            custom_links: links.into_iter().map(Into::into).collect(),
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
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            image_url,
            image_link_url,
            links,
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
            contact_name,
            email,
            telephone,
            homepage,
            opening_hours,
            founded_on,
            categories,
            tags,
            image_url,
            image_link_url,
            custom_links: links.into_iter().map(Into::into).collect(),
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
        founded_on,
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
        name: contact_name,
        email,
        phone: telephone,
    } = contact.unwrap_or_default();

    let (homepage_url, image_url, image_link_url, custom_links) = links
        .map(
            |e::Links {
                 homepage,
                 image,
                 image_href,
                 custom,
             }| (homepage, image, image_href, custom),
        )
        .unwrap_or_default();

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
        contact_name,
        email: email.map(Into::into),
        telephone,
        homepage: homepage_url.map(Url::into_string),
        opening_hours: opening_hours.map(Into::into),
        founded_on: founded_on.map(Into::into),
        categories: categories.into_iter().map(|c| c.id.to_string()).collect(),
        tags,
        ratings: ratings.into_iter().map(|r| r.id.to_string()).collect(),
        license: Some(license),
        image_url: image_url.map(Url::into_string),
        image_link_url: image_link_url.map(Url::into_string),
        custom_links: custom_links.into_iter().map(Into::into).collect(),
    }
}

#[derive(Debug, Deserialize)]
pub struct Review {
    pub status: ReviewStatus,
    pub comment: Option<String>,
}
