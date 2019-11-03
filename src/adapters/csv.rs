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

impl From<(PlaceRev, Vec<Category>, AvgRatingValue)> for CsvRecord {
    fn from(t: (PlaceRev, Vec<Category>, AvgRatingValue)) -> Self {
        let (e, categories, avg_rating) = t;

        let PlaceRev {
            uid,
            created,
            revision,
            title,
            description,
            location,
            homepage,
            license,
            image_url,
            image_link_url,
            ..
        } = e.clone();

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
            .map(|c| c.tag)
            .collect::<Vec<_>>()
            .join(",");

        CsvRecord {
            id: uid.into(),
            created: created.when.into(),
            version: revision.into(),
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
            tags: e.tags.join(","),
            avg_rating: avg_rating.into(),
        }
    }
}
