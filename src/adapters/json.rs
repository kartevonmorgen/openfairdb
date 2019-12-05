use crate::core::{db::IndexedPlace, entities as e, util::geo::MapPoint};
pub use ofdb_boundary::*;
use url::Url;

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
            id: id.into(),
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

impl From<Coordinate> for MapPoint {
    fn from(c: Coordinate) -> Self {
        MapPoint::try_from_lat_lng_deg(c.lat, c.lng).unwrap_or_default()
    }
}

impl From<e::RatingContext> for RatingContext {
    fn from(from: e::RatingContext) -> Self {
        use e::RatingContext as E;
        use RatingContext as C;
        match from {
            E::Diversity => C::Diversity,
            E::Renewable => C::Renewable,
            E::Fairness => C::Fairness,
            E::Humanity => C::Humanity,
            E::Transparency => C::Transparency,
            E::Solidarity => C::Solidarity,
        }
    }
}

impl From<e::Category> for Category {
    fn from(from: e::Category) -> Self {
        let name = from.name();
        Self {
            id: from.id.into(),
            name,
        }
    }
}

impl From<IndexedPlace> for EntrySearchResult {
    fn from(from: IndexedPlace) -> Self {
        let IndexedPlace {
            id,
            title,
            description,
            tags,
            pos,
            ratings,
            ..
        } = from;
        let (tags, categories) = e::Category::split_from_tags(tags);
        let categories = categories.into_iter().map(|c| c.id.to_string()).collect();
        let lat = pos.lat().to_deg();
        let lng = pos.lng().to_deg();
        let e::AvgRatings {
            diversity,
            fairness,
            humanity,
            renewable,
            solidarity,
            transparency,
        } = ratings;
        let total = ratings.total().into();
        let ratings = EntrySearchRatings {
            total,
            diversity: diversity.into(),
            fairness: fairness.into(),
            humanity: humanity.into(),
            renewable: renewable.into(),
            solidarity: solidarity.into(),
            transparency: transparency.into(),
        };
        Self {
            id,
            lat,
            lng,
            title,
            description,
            categories,
            tags,
            ratings,
        }
    }
}

impl From<e::User> for User {
    fn from(from: e::User) -> Self {
        let e::User {
            email,
            email_confirmed,
            role,
            password: _password,
        } = from;
        Self {
            email,
            email_confirmed,
            role: role.into(),
        }
    }
}

impl From<e::Role> for UserRole {
    fn from(from: e::Role) -> Self {
        use e::Role::*;
        match from {
            Guest => UserRole::Guest,
            User => UserRole::User,
            Scout => UserRole::Scout,
            Admin => UserRole::Admin,
        }
    }
}

impl From<UserRole> for e::Role {
    fn from(from: UserRole) -> Self {
        use e::Role::*;
        match from {
            UserRole::Guest => Guest,
            UserRole::User => User,
            UserRole::Scout => Scout,
            UserRole::Admin => Admin,
        }
    }
}

// Entity -> JSON

pub fn entry_from_place_with_ratings(place: e::Place, ratings: Vec<e::Rating>) -> Entry {
    let e::Place {
        id,
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
        id: id.into(),
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
        categories: categories.into_iter().map(|c| c.id.to_string()).collect(),
        tags,
        ratings: ratings.into_iter().map(|r| r.id.to_string()).collect(),
        license: Some(license),
        image_url: image_url.map(Url::into_string),
        image_link_url: image_link_url.map(Url::into_string),
    }
}

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
    pub comment: Option<String>,
}

impl From<e::ActivityLog> for ActivityLog {
    fn from(from: e::ActivityLog) -> Self {
        let e::ActivityLog {
            activity: e::Activity { at, by },
            context: ctx,
            comment,
        } = from;
        Self {
            at: at.into_inner(),
            by: by.map(Into::into),
            ctx,
            comment,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceRoot {
    pub id: String,

    #[serde(rename = "lic")]
    pub license: String,
}

impl From<e::PlaceRoot> for PlaceRoot {
    fn from(from: e::PlaceRoot) -> Self {
        let e::PlaceRoot { id, license } = from;
        Self {
            id: id.into(),
            license,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceRevision {
    #[serde(rename = "rev")]
    pub revision: u64,

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

impl From<e::PlaceRevision> for PlaceRevision {
    fn from(from: e::PlaceRevision) -> Self {
        let e::PlaceRevision {
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

impl From<ReviewStatus> for e::ReviewStatus {
    fn from(from: ReviewStatus) -> Self {
        match from {
            ReviewStatus::Rejected => e::ReviewStatus::Rejected,
            ReviewStatus::Archived => e::ReviewStatus::Archived,
            ReviewStatus::Created => e::ReviewStatus::Created,
            ReviewStatus::Confirmed => e::ReviewStatus::Confirmed,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Review {
    pub status: ReviewStatus,
    pub comment: Option<String>,
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
    pub revisions: Vec<(PlaceRevision, Vec<ReviewStatusLog>)>,
}

impl From<e::PlaceHistory> for PlaceHistory {
    fn from(from: e::PlaceHistory) -> Self {
        let e::PlaceHistory { place, revisions } = from;
        Self {
            place: place.into(),
            revisions: revisions
                .into_iter()
                .map(|(place_revision, reviews)| {
                    (
                        place_revision.into(),
                        reviews.into_iter().map(Into::into).collect(),
                    )
                })
                .collect(),
        }
    }
}

impl From<e::AvgRatingValue> for AvgRatingValue {
    fn from(v: e::AvgRatingValue) -> Self {
        let v: f64 = v.into();
        AvgRatingValue::from(v)
    }
}

impl From<e::RatingValue> for RatingValue {
    fn from(v: e::RatingValue) -> Self {
        let v: i8 = v.into();
        RatingValue::from(v)
    }
}
