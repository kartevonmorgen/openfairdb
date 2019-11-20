use crate::core::{db::IndexedPlace, entities as e, util::geo::MapPoint};

use url::Url;

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
            homepage: homepage.map(Url::into_string),
            tags,
            registration,
            organizer,
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
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

impl From<IndexedPlace> for EntrySearchResult {
    fn from(from: IndexedPlace) -> Self {
        let (tags, categories) = e::Category::split_from_tags(from.tags);
        Self {
            id: from.id,
            lat: from.pos.lat().to_deg(),
            lng: from.pos.lng().to_deg(),
            title: from.title,
            description: from.description,
            categories: categories.into_iter().map(|c| c.uid.to_string()).collect(),
            tags,
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
    pub fn from_entry_with_ratings(place: e::Place, ratings: Vec<e::Rating>) -> Entry {
        let e::Place {
            uid,
            license,
            revision,
            created,
            title,
            description,
            location,
            contact,
            links,
            tags,
        } = place;

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
        } = contact.unwrap_or_default();

        let (homepage_url, image_url, image_link_url) = if let Some(links) = links {
            (links.homepage, links.image, links.image_href)
        } else {
            (None, None, None)
        };

        let (tags, categories) = e::Category::split_from_tags(tags);

        Entry {
            id: uid.into(),
            created: created.when.into_seconds(),
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
            homepage: homepage_url.map(Url::into_string),
            categories: categories.into_iter().map(|c| c.uid.to_string()).collect(),
            tags,
            ratings: ratings.into_iter().map(|r| r.uid.to_string()).collect(),
            license: Some(license),
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Activity {
    pub when: i64,
    pub who: Option<String>,
}

impl From<e::Activity> for Activity {
    fn from(from: e::Activity) -> Self {
        Self {
            when: from.when.into_inner(),
            who: from.who.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LatLonDegrees(f64, f64);

impl From<MapPoint> for LatLonDegrees {
    fn from(from: MapPoint) -> Self {
        Self(from.lat().to_deg(), from.lng().to_deg())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

impl From<e::Address> for Address {
    fn from(from: e::Address) -> Self {
        let e::Address {
            street,
            zip,
            city,
            country,
        } = from;
        Self {
            street,
            zip,
            city,
            country,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub coords: LatLonDegrees,
    pub address: Option<Address>,
}

impl From<e::Location> for Location {
    fn from(from: e::Location) -> Self {
        let e::Location { pos, address } = from;
        Self {
            coords: pos.into(),
            address: address.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub email: Option<String>,
    pub phone: Option<String>,
}

impl From<e::Contact> for Contact {
    fn from(from: e::Contact) -> Self {
        let e::Contact { email, phone } = from;
        Self {
            email: email.map(Into::into),
            phone,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub home: Option<Url>,
    pub img: Option<Url>,
    pub img_href: Option<Url>,
}

impl From<e::Links> for Links {
    fn from(from: e::Links) -> Self {
        let e::Links {
            homepage: home,
            image: img,
            image_href: img_href,
        } = from;
        Self {
            home,
            img,
            img_href,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceRoot {
    pub uid: String,
    pub lic: String,
}

impl From<e::PlaceRoot> for PlaceRoot {
    fn from(from: e::PlaceRoot) -> Self {
        let e::PlaceRoot { uid, license: lic } = from;
        Self {
            uid: uid.into(),
            lic,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceState {
    pub rev: u64,
    pub created: Activity,
    pub title: String,
    pub desc: String,
    pub loc: Location,
    pub contact: Option<Contact>,
    pub links: Option<Links>,
    pub tag: Vec<String>,
}

impl From<e::PlaceState> for PlaceState {
    fn from(from: e::PlaceState) -> Self {
        let e::PlaceState {
            revision,
            created,
            title,
            description: desc,
            location,
            contact,
            links,
            tags: tag,
        } = from;
        Self {
            rev: revision.into(),
            created: created.into(),
            title,
            desc,
            loc: location.into(),
            contact: contact.map(Into::into),
            links: links.map(Into::into),
            tag,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReviewStatus {
    Rejected,
    Archived,
    Created,
    Confirmed,
}

impl From<e::ReviewStatus> for ReviewStatus {
    fn from(from: e::ReviewStatus) -> Self {
        match from {
            e::ReviewStatus::Rejected => ReviewStatus::Rejected,
            e::ReviewStatus::Archived => ReviewStatus::Archived,
            e::ReviewStatus::Created => ReviewStatus::Created,
            e::ReviewStatus::Confirmed => ReviewStatus::Confirmed,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewStatusLog {
    pub activity: Activity,
    pub status: ReviewStatus,
    pub context: Option<String>,
    pub notes: Option<String>,
}
