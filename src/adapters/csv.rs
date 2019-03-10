use crate::core::entities::*;

#[derive(Debug, Serialize)]
pub struct CsvRecord {
    pub id: String,
    pub osm_node: Option<u64>,
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
    pub license: Option<String>,
    pub avg_rating: f64,
}

impl From<(Entry, Vec<Category>, AvgRatingValue)> for CsvRecord {
    fn from(t: (Entry, Vec<Category>, AvgRatingValue)) -> Self {
        let (e, categories, avg_rating) = t;

        let Entry {
            id,
            osm_node,
            created,
            version,
            title,
            description,
            location,
            homepage,
            license,
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
            .map(|c| c.name)
            .collect::<Vec<_>>()
            .join(",");

        CsvRecord {
            id,
            osm_node: osm_node.map(|x| x as u64),
            created: created.into(),
            version: version as u64,
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
            categories,
            tags: e.tags.join(","),
            avg_rating: avg_rating.into(),
        }
    }
}
