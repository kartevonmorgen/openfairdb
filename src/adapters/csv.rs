use crate::core::entities::*;

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
            uid,
            rev,
            created,
            license,
            title,
            description,
            location,
            homepage,
            image_url,
            image_link_url,
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

        let categories = categories
            .into_iter()
            .map(|c| c.uid)
            .collect::<Vec<_>>()
            .join(",");

        CsvRecord {
            id: uid.into(),
            created: created.when.into(),
            version: rev.into(),
            title,
            description,
            lat: pos.lat().to_deg(),
            lng: pos.lng().to_deg(),
            street,
            zip,
            city,
            country,
            homepage,
            license,
            image_url,
            image_link_url,
            categories,
            tags: tags.join(","),
            avg_rating: avg_rating.into(),
        }
    }
}
