use crate::core::{db::IndexedEntry, entities as e, util::geo::MapPoint};
pub use ofdb_boundary::*;

impl From<e::Event> for Event {
    fn from(e: e::Event) -> Self {
        let e::Event {
            id,
            title,
            description,
            start,
            end,
            location,
            contact,
            tags,
            homepage,
            registration,
            organizer,
            image_url,
            image_link_url,
            ..
        } = e;

        let (lat, lng, address) = if let Some(location) = location {
            if location.pos.is_valid() {
                let lat = location.pos.lat().to_deg();
                let lng = location.pos.lng().to_deg();
                (Some(lat), Some(lng), location.address)
            } else {
                (None, None, location.address)
            }
        } else {
            (None, None, None)
        };

        let e::Address {
            street,
            zip,
            city,
            country,
        } = address.unwrap_or_default();

        let e::Contact { email, telephone } = contact.unwrap_or_default();

        let registration = registration.map(|r| {
            match r {
                e::RegistrationType::Email => "email",
                e::RegistrationType::Phone => "telephone",
                e::RegistrationType::Homepage => "homepage",
            }
            .to_string()
        });

        let start = start.timestamp();
        let end = end.map(|end| end.timestamp());

        Event {
            id,
            // created,
            title,
            description,
            start,
            end,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            email,
            telephone,
            homepage,
            tags,
            registration,
            organizer,
            image_url,
            image_link_url,
        }
    }
}

impl From<Coordinate> for MapPoint {
    fn from(c: Coordinate) -> Self {
        MapPoint::try_from_lat_lng_deg(c.lat, c.lng).unwrap_or_default()
    }
}

impl From<e::RatingContext> for RatingContext {
    fn from(r: e::RatingContext) -> Self {
        use e::RatingContext as E;
        use RatingContext as C;
        match r {
            E::Diversity => C::Diversity,
            E::Renewable => C::Renewable,
            E::Fairness => C::Fairness,
            E::Humanity => C::Humanity,
            E::Transparency => C::Transparency,
            E::Solidarity => C::Solidarity,
        }
    }
}

impl From<IndexedEntry> for EntrySearchResult {
    fn from(from: IndexedEntry) -> Self {
        Self {
            id: from.id,
            lat: from.pos.lat().to_deg(),
            lng: from.pos.lng().to_deg(),
            title: from.title,
            description: from.description,
            categories: from.categories,
            tags: from.tags,
            ratings: EntrySearchRatings {
                total: f64::from(from.ratings.total()).into(),
                diversity: f64::from(from.ratings.diversity).into(),
                fairness: f64::from(from.ratings.fairness).into(),
                humanity: f64::from(from.ratings.humanity).into(),
                renewable: f64::from(from.ratings.renewable).into(),
                solidarity: f64::from(from.ratings.solidarity).into(),
                transparency: f64::from(from.ratings.transparency).into(),
            },
        }
    }
}

// Entity -> JSON

pub fn from_entry_with_ratings(e: e::Entry, ratings: Vec<e::Rating>) -> Entry {
    let e::Entry {
        id,
        created,
        version,
        title,
        description,
        location,
        homepage,
        categories,
        tags,
        license,
        image_url,
        image_link_url,
        ..
    } = e;
    let e::Location { pos, address } = location;
    let lat = pos.lat().to_deg();
    let lng = pos.lng().to_deg();
    let e::Address {
        street,
        zip,
        city,
        country,
    } = address.unwrap_or_default();

    let e::Contact { email, telephone } = e.contact.unwrap_or_default();

    Entry {
        id,
        created: created.into(),
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
        homepage,
        categories,
        tags,
        ratings: ratings.into_iter().map(|r| r.id).collect(),
        license,
        image_url,
        image_link_url,
    }
}

impl From<e::TagFrequency> for TagFrequency {
    fn from(from: e::TagFrequency) -> Self {
        Self(from.0, from.1)
    }
}
