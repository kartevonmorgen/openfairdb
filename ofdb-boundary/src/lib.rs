use ofdb_entities as e;
use serde::{Deserialize, Serialize};
use url::Url;

#[rustfmt::skip]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq))]
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
    pub state          : Option<String>,
    pub email          : Option<String>,
    pub telephone      : Option<String>,
    pub homepage       : Option<String>,
    pub opening_hours  : Option<String>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub ratings        : Vec<String>,
    pub license        : Option<String>,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
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
    pub state: Option<String>,
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

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy, PartialEq))]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct User {
    pub email: String,
    pub email_confirmed: bool,
    pub role: UserRole,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy))]
pub struct RatingValue(i8);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, Copy))]
pub struct AvgRatingValue(f64);

impl From<f64> for AvgRatingValue {
    fn from(v: f64) -> Self {
        Self(v)
    }
}

impl From<AvgRatingValue> for f64 {
    fn from(from: AvgRatingValue) -> Self {
        from.0
    }
}

impl From<i8> for RatingValue {
    fn from(from: i8) -> Self {
        Self(from)
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "snake_case")]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct EntrySearchRatings {
    pub total: AvgRatingValue,
    pub diversity: AvgRatingValue,
    pub fairness: AvgRatingValue,
    pub humanity: AvgRatingValue,
    pub renewable: AvgRatingValue,
    pub solidarity: AvgRatingValue,
    pub transparency: AvgRatingValue,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Comment {
    pub id: String,
    pub created: i64,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Category {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct PlaceSearchResult {
    pub id: String,
    pub status: Option<ReviewStatus>,
    pub lat: f64,
    pub lng: f64,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: EntrySearchRatings,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Archived,
    Confirmed,
    Created,
    Rejected,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct SearchResponse {
    pub visible: Vec<PlaceSearchResult>,
    pub invisible: Vec<PlaceSearchResult>,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(
    feature = "extra-derive",
    derive(Debug, Clone, Copy, PartialEq, Eq, Hash)
)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Guest,
    User,
    Scout,
    Admin,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct RequestPasswordReset {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct ResetPassword {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct TagFrequency(pub String, pub u64);

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct Rating {
    pub id: String,
    pub title: String,
    pub created: i64,
    pub value: RatingValue,
    pub context: RatingContext,
    pub comments: Vec<Comment>,
    pub source: String,
}

impl From<e::category::Category> for Category {
    fn from(from: e::category::Category) -> Self {
        let name = from.name();
        Self {
            id: from.id.into(),
            name,
        }
    }
}

impl From<e::review::ReviewStatus> for ReviewStatus {
    fn from(from: e::review::ReviewStatus) -> Self {
        use e::review::ReviewStatus::*;
        match from {
            Archived => ReviewStatus::Archived,
            Confirmed => ReviewStatus::Confirmed,
            Created => ReviewStatus::Created,
            Rejected => ReviewStatus::Rejected,
        }
    }
}

impl From<ReviewStatus> for e::review::ReviewStatus {
    fn from(from: ReviewStatus) -> Self {
        use e::review::ReviewStatus::*;
        match from {
            ReviewStatus::Archived => Archived,
            ReviewStatus::Confirmed => Confirmed,
            ReviewStatus::Created => Created,
            ReviewStatus::Rejected => Rejected,
        }
    }
}

impl From<e::user::User> for User {
    fn from(from: e::user::User) -> Self {
        let e::user::User {
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

impl From<e::user::Role> for UserRole {
    fn from(from: e::user::Role) -> Self {
        use e::user::Role::*;
        match from {
            Guest => UserRole::Guest,
            User => UserRole::User,
            Scout => UserRole::Scout,
            Admin => UserRole::Admin,
        }
    }
}

impl From<UserRole> for e::user::Role {
    fn from(from: UserRole) -> Self {
        use e::user::Role::*;
        match from {
            UserRole::Guest => Guest,
            UserRole::User => User,
            UserRole::Scout => Scout,
            UserRole::Admin => Admin,
        }
    }
}

impl From<Coordinate> for e::geo::MapPoint {
    fn from(c: Coordinate) -> Self {
        e::geo::MapPoint::try_from_lat_lng_deg(c.lat, c.lng).unwrap_or_default()
    }
}

impl From<e::tag::TagFrequency> for TagFrequency {
    fn from(from: e::tag::TagFrequency) -> Self {
        Self(from.0, from.1)
    }
}

impl From<e::rating::RatingContext> for RatingContext {
    fn from(from: e::rating::RatingContext) -> Self {
        use e::rating::RatingContext as E;
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

impl From<RatingContext> for e::rating::RatingContext {
    fn from(from: RatingContext) -> Self {
        use e::rating::RatingContext as E;
        use RatingContext as C;
        match from {
            C::Diversity => E::Diversity,
            C::Renewable => E::Renewable,
            C::Fairness => E::Fairness,
            C::Humanity => E::Humanity,
            C::Transparency => E::Transparency,
            C::Solidarity => E::Solidarity,
        }
    }
}

impl From<e::rating::AvgRatingValue> for AvgRatingValue {
    fn from(v: e::rating::AvgRatingValue) -> Self {
        let v: f64 = v.into();
        AvgRatingValue::from(v)
    }
}

impl From<e::rating::RatingValue> for RatingValue {
    fn from(v: e::rating::RatingValue) -> Self {
        let v: i8 = v.into();
        RatingValue::from(v)
    }
}

impl From<RatingValue> for e::rating::RatingValue {
    fn from(v: RatingValue) -> Self {
        e::rating::RatingValue::from(v.0)
    }
}

impl From<e::event::Event> for Event {
    fn from(e: e::event::Event) -> Self {
        let e::event::Event {
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

        let e::address::Address {
            street,
            zip,
            city,
            country,
            state,
        } = address.unwrap_or_default();

        let e::contact::Contact {
            email,
            phone: telephone,
        } = contact.unwrap_or_default();

        let registration = registration.map(|r| {
            match r {
                e::event::RegistrationType::Email => "email",
                e::event::RegistrationType::Phone => "telephone",
                e::event::RegistrationType::Homepage => "homepage",
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
            state,
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
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct ReviewedRevision {
    pub revision: u64,
    pub review_status: Option<ReviewStatus>,
}

impl From<e::authorization::ReviewedRevision> for ReviewedRevision {
    fn from(from: e::authorization::ReviewedRevision) -> Self {
        let e::authorization::ReviewedRevision {
            revision,
            review_status,
        } = from;
        Self {
            revision: revision.into(),
            review_status: review_status.map(Into::into),
        }
    }
}

impl From<ReviewedRevision> for e::authorization::ReviewedRevision {
    fn from(from: ReviewedRevision) -> Self {
        let ReviewedRevision {
            revision,
            review_status,
        } = from;
        Self {
            revision: revision.into(),
            review_status: review_status.map(Into::into),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct PendingAuthorizationForPlace {
    pub place_id: String,
    pub created_at: i64,
    pub last_authorized: Option<ReviewedRevision>,
}

impl From<e::authorization::PendingAuthorizationForPlace> for PendingAuthorizationForPlace {
    fn from(from: e::authorization::PendingAuthorizationForPlace) -> Self {
        let e::authorization::PendingAuthorizationForPlace {
            place_id,
            created_at,
            last_authorized,
        } = from;
        Self {
            place_id: place_id.into(),
            created_at: created_at.into_inner(),
            last_authorized: last_authorized.map(Into::into),
        }
    }
}

#[derive(Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct AuthorizationForPlace {
    pub place_id: String,
    pub authorized: Option<ReviewedRevision>,
}

impl From<AuthorizationForPlace> for e::authorization::AuthorizationForPlace {
    fn from(from: AuthorizationForPlace) -> Self {
        let AuthorizationForPlace {
            place_id,
            authorized,
        } = from;
        Self {
            place_id: place_id.into(),
            authorized: authorized.map(Into::into),
        }
    }
}

#[derive(Serialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone, PartialEq, Eq))]
pub struct ResultCount {
    pub count: u64,
}
