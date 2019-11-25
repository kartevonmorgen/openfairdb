use crate::core::entities::*;

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
