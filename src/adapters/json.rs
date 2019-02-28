use crate::core::{db::IndexedEntry, entities as e, util::geo::MapPoint};

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
    pub start: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
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
    pub registration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizer: Option<String>,
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
            registration,
            organizer,
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
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

impl From<Coordinate> for MapPoint {
    fn from(c: Coordinate) -> Self {
        MapPoint::try_from_lat_lng_deg(c.lat, c.lng).unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Rating {
    pub id: String,
    pub title: String,
    pub created: u64,
    pub value: e::RatingValue,
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
pub struct EntrySearchRatings {
    pub total: e::AvgRatingValue,
    pub diversity: e::AvgRatingValue,
    pub fairness: e::AvgRatingValue,
    pub humanity: e::AvgRatingValue,
    pub renewable: e::AvgRatingValue,
    pub solidarity: e::AvgRatingValue,
    pub transparency: e::AvgRatingValue,
}

#[derive(Serialize)]
pub struct EntrySearchResult {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: EntrySearchRatings,
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
                total: from.ratings.total(),
                diversity: from.ratings.diversity,
                fairness: from.ratings.fairness,
                humanity: from.ratings.humanity,
                renewable: from.ratings.renewable,
                solidarity: from.ratings.solidarity,
                transparency: from.ratings.transparency,
            },
        }
    }
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub visible: Vec<EntrySearchResult>,
    pub invisible: Vec<EntrySearchResult>,
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
