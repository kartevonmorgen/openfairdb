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
            created: created.at.into_seconds(),
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LatLonDegrees(f64, f64);

impl From<MapPoint> for LatLonDegrees {
    fn from(from: MapPoint) -> Self {
        Self(from.lat().to_deg(), from.lng().to_deg())
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

impl Address {
    pub fn is_empty(&self) -> bool {
        self.street.is_none() && self.zip.is_none() && self.city.is_none() && self.country.is_none()
    }
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "deg")]
    pub latlon: LatLonDegrees,

    #[serde(
        rename = "adr",
        skip_serializing_if = "Address::is_empty",
        default = "Default::default"
    )]
    pub address: Address,
}

impl From<e::Location> for Location {
    fn from(from: e::Location) -> Self {
        let e::Location { pos, address } = from;
        Self {
            latlon: pos.into(),
            address: address.map(Into::into).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.phone.is_none()
    }
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

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "www", skip_serializing_if = "Option::is_none")]
    pub homepage: Option<Url>,

    #[serde(rename = "img", skip_serializing_if = "Option::is_none")]
    pub image: Option<Url>,

    #[serde(rename = "img_href", skip_serializing_if = "Option::is_none")]
    pub image_href: Option<Url>,
}

impl Links {
    pub fn is_empty(&self) -> bool {
        self.homepage.is_none() && self.image.is_none() && self.image_href.is_none()
    }
}

impl From<e::Links> for Links {
    fn from(from: e::Links) -> Self {
        let e::Links {
            homepage,
            image,
            image_href,
        } = from;
        Self {
            homepage,
            image,
            image_href,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
}

impl From<e::Activity> for Activity {
    fn from(from: e::Activity) -> Self {
        let e::Activity { at, by } = from;
        Self {
            at: at.into_inner(),
            by: by.map(Into::into),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityLog {
    pub at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctx: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl From<e::ActivityLog> for ActivityLog {
    fn from(from: e::ActivityLog) -> Self {
        let e::ActivityLog {
            activity: e::Activity { at, by },
            context: ctx,
            memo,
        } = from;
        Self {
            at: at.into_inner(),
            by: by.map(Into::into),
            ctx,
            memo,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceRoot {
    pub uid: String,

    #[serde(rename = "lic")]
    pub license: String,
}

impl From<e::PlaceRoot> for PlaceRoot {
    fn from(from: e::PlaceRoot) -> Self {
        let e::PlaceRoot { uid, license } = from;
        Self {
            uid: uid.into(),
            license,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceState {
    #[serde(rename = "rev")]
    pub revision: u64,

    #[serde(rename = "act")]
    pub created: Activity,

    #[serde(rename = "tit")]
    pub title: String,

    #[serde(rename = "dsc")]
    pub description: String,

    #[serde(rename = "loc")]
    pub location: Location,

    #[serde(
        rename = "cnt",
        skip_serializing_if = "Contact::is_empty",
        default = "Default::default"
    )]
    pub contact: Contact,

    #[serde(
        rename = "lnk",
        skip_serializing_if = "Links::is_empty",
        default = "Default::default"
    )]
    pub links: Links,

    #[serde(
        rename = "tag",
        skip_serializing_if = "Vec::is_empty",
        default = "Default::default"
    )]
    pub tags: Vec<String>,
}

impl From<e::PlaceState> for PlaceState {
    fn from(from: e::PlaceState) -> Self {
        let e::PlaceState {
            revision,
            created,
            title,
            description,
            location,
            contact,
            links,
            tags,
        } = from;
        Self {
            revision: revision.into(),
            created: created.into(),
            title,
            description,
            location: location.into(),
            contact: contact.map(Into::into).unwrap_or_default(),
            links: links.map(Into::into).unwrap_or_default(),
            tags,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    pub rev: u64,
    pub act: ActivityLog,
    pub status: ReviewStatus,
}

impl From<e::ReviewStatusLog> for ReviewStatusLog {
    fn from(from: e::ReviewStatusLog) -> Self {
        let e::ReviewStatusLog {
            revision,
            activity,
            status,
        } = from;
        Self {
            rev: revision.into(),
            act: activity.into(),
            status: status.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceHistory {
    pub place: PlaceRoot,
    pub revisions: Vec<(PlaceState, Vec<ReviewStatusLog>)>,
}

impl From<e::PlaceHistory> for PlaceHistory {
    fn from(from: e::PlaceHistory) -> Self {
        let e::PlaceHistory { place, revisions } = from;
        Self {
            place: place.into(),
            revisions: revisions
                .into_iter()
                .map(|(place_state, reviews)| {
                    (
                        place_state.into(),
                        reviews.into_iter().map(Into::into).collect(),
                    )
                })
                .collect(),
        }
    }
}
