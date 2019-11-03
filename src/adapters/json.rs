use crate::core::{db::IndexedEntry, entities as e, util::geo::MapPoint};

#[rustfmt::skip]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id             : String,
    pub created        : i64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_link_url: Option<String>,
}

impl From<e::Event> for Event {
    fn from(e: e::Event) -> Self {
        let e::Event {
            uid,
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

        let e::Contact {
            email,
            phone: telephone,
        } = contact.unwrap_or_default();

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
            id: uid.into(),
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
            email: email.map(Into::into),
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
    pub created: i64,
    pub value: e::RatingValue,
    pub context: e::RatingContext,
    pub comments: Vec<Comment>,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub created: i64,
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
pub struct Category {
    pub id: String,
    pub name: String,
}

impl From<e::Category> for Category {
    fn from(from: e::Category) -> Self {
        let name = from.name();
        Self {
            id: from.uid.into(),
            name,
        }
    }
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
    pub email: String,
}

impl From<e::User> for User {
    fn from(from: e::User) -> Self {
        let e::User { email, .. } = from;
        Self { email }
    }
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
    pub fn from_entry_with_ratings(e: e::PlaceRev, ratings: Vec<e::Rating>) -> Entry {
        let e::PlaceRev {
            uid,
            created,
            revision,
            title,
            description,
            location,
            homepage,
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

        let e::Contact {
            email,
            phone: telephone,
        } = e.contact.unwrap_or_default();

        let (tags, categories) = e::Category::split_from_tags(tags);

        Entry {
            id: uid.into(),
            created: created.when.into(),
            version: revision.into(),
            title,
            description,
            lat,
            lng,
            street,
            zip,
            city,
            country,
            email: email.map(Into::into),
            telephone,
            homepage,
            categories: categories.into_iter().map(|c| c.uid.to_string()).collect(),
            tags,
            ratings: ratings.into_iter().map(|r| r.uid.to_string()).collect(),
            license: Some(license),
            image_url,
            image_link_url,
        }
    }
}

#[derive(Deserialize)]
pub struct RequestPasswordReset {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPassword {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct TagFrequency(pub String, pub u64);

impl From<e::TagFrequency> for TagFrequency {
    fn from(from: e::TagFrequency) -> Self {
        Self(from.0, from.1)
    }
}
