use crate::core::{entities::*, util::time::Timestamp};

use url::Url;

#[derive(Debug, Serialize)]
pub struct CsvRecord {
    pub id: String,
    pub created: i64,
    pub version: u64,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lng: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub homepage: Option<String>,
    pub categories: String,
    pub tags: String,
    pub license: String,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    pub avg_rating: f64,
}

impl From<(Place, Vec<Category>, AvgRatingValue)> for CsvRecord {
    fn from(from: (Place, Vec<Category>, AvgRatingValue)) -> Self {
        let (place, categories, avg_rating) = from;

        let Place {
            id,
            license,
            revision,
            created,
            title,
            description,
            location,
            links,
            tags,
            ..
        } = place;

        let Location { pos, address } = location;

        let address = address.unwrap_or_default();

        let Address {
            street,
            zip,
            city,
            country,
        } = address;

        let (homepage_url, image_url, image_link_url) = if let Some(links) = links {
            (links.homepage, links.image, links.image_href)
        } else {
            (None, None, None)
        };

        let categories = categories
            .into_iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
            .join(",");

        CsvRecord {
            id: id.into(),
            created: created.at.into_seconds(),
            version: revision.into(),
            title,
            description,
            lat: pos.lat().to_deg(),
            lng: pos.lng().to_deg(),
            street,
            zip,
            city,
            country,
            homepage: homepage_url.map(Url::into_string),
            license,
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
            categories,
            tags: tags.join(","),
            avg_rating: avg_rating.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventRecord {
    pub id: String,
    pub created_by: Option<String>,
    pub organizer: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub start: i64,
    pub end: Option<i64>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    pub tags: String,
}

impl From<Event> for EventRecord {
    fn from(from: Event) -> Self {
        let Event {
            id,
            created_by,
            organizer,
            title,
            description,
            start,
            end,
            location,
            contact,
            homepage,
            image_url,
            image_link_url,
            tags,
            ..
        } = from;

        let (pos, address) = location.map_or((None, None), |l| (Some(l.pos), l.address));

        let (lat, lng) = pos.map_or((None, None), |p| {
            (Some(p.lat().to_deg()), Some(p.lng().to_deg()))
        });

        let address = address.unwrap_or_default();
        let Address {
            street,
            zip,
            city,
            country,
        } = address;

        let Contact { email, phone } = contact.unwrap_or_default();

        Self {
            id: id.into(),
            created_by,
            title,
            description,
            start: Timestamp::from(start).into_seconds(),
            end: end.map(|end| Timestamp::from(end).into_seconds()),
            lat,
            lng,
            street,
            zip,
            city,
            country,
            organizer,
            email: email.map(Into::into),
            phone,
            homepage: homepage.map(Url::into_string),
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
            tags: tags.join(","),
        }
    }
}
