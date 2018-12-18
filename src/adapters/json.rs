use crate::core::entities as e;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id             : String,
    pub created        : u64,
    pub version        : u64,
    pub title          : String,
    pub description    : String,
    pub lat            : f64,
    pub lng            : f64,
    pub street         : Option<String>,
    pub zip            : Option<String>,
    pub city           : Option<String>,
    pub country        : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub ratings        : Vec<String>,
    pub license        : Option<String>,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
    pub lat: f64,
    pub lng: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telephone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

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
            created_by,
        } = e;

        let e::Location { lat, lng, address } = location.unwrap_or_default();

        let e::Address {
            street,
            zip,
            city,
            country,
        } = address.unwrap_or_default();

        let e::Contact { email, telephone } = contact.unwrap_or_default();

        Event {
            id,
            //created      ,
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
            created_by,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

impl From<Coordinate> for e::Coordinate {
    fn from(c: Coordinate) -> Self {
        e::Coordinate {
            lat: c.lat,
            lng: c.lng,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Rating {
    pub id: String,
    pub title: String,
    pub created: u64,
    pub value: i8,
    pub context: e::RatingContext,
    pub comments: Vec<Comment>,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub created: u64,
    pub text: String,
}

#[derive(Serialize)]
pub struct EntryIdWithCoordinates {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
}

impl From<e::Entry> for EntryIdWithCoordinates {
    fn from(e: e::Entry) -> Self {
        EntryIdWithCoordinates {
            id: e.id,
            lat: e.location.lat,
            lng: e.location.lng,
        }
    }
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub visible: Vec<EntryIdWithCoordinates>,
    pub invisible: Vec<EntryIdWithCoordinates>,
}

#[derive(Serialize)]
pub struct User {
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
}

// Entity -> JSON

impl Entry {
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
        let e::Location { lat, lng, address } = location;
        let e::Address {
            street,
            zip,
            city,
            country,
        } = address.unwrap_or_default();

        let e::Contact { email, telephone } = e.contact.unwrap_or_default();

        Entry {
            id,
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
}
