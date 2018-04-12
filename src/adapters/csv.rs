use core::entities::Entry;

#[derive(Debug, Serialize)]
pub struct CsvRecord {
    id: String,
    osm_node: Option<u64>,
    created: u64,
    version: u64,
    title: String,
    description: String,
    lat: f64,
    lng: f64,
    street: Option<String>,
    zip: Option<String>,
    city: Option<String>,
    country: Option<String>,
    homepage: Option<String>,
    categories: String,
    tags: String,
    license: Option<String>,
}

impl From<Entry> for CsvRecord {
    fn from(e: Entry) -> Self {
        let Entry {
            id,
            osm_node,
            created,
            version,
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            homepage,
            license,
            ..
        } = e;

        let categories = e.categories.join(",");
        let tags = e.tags.join(",");

        CsvRecord {
            id,
            osm_node: osm_node.map(|x| x as u64),
            created: created as u64,
            version: version as u64,
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            homepage,
            license,
            categories,
            tags
        }
    }
}
