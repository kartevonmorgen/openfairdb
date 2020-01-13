use ofdb_core as e;
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

impl From<Coordinate> for e::geo::MapPoint {
    fn from(c: Coordinate) -> Self {
        e::geo::MapPoint::try_from_lat_lng_deg(c.lat, c.lng).unwrap_or_default()
    }
}

impl From<e::ReviewStatus> for ReviewStatus {
    fn from(from: e::ReviewStatus) -> Self {
        use e::ReviewStatus::*;
        match from {
            Archived => ReviewStatus::Archived,
            Confirmed => ReviewStatus::Confirmed,
            Created => ReviewStatus::Created,
            Rejected => ReviewStatus::Rejected,
        }
    }
}

impl From<ReviewStatus> for e::ReviewStatus {
    fn from(from: ReviewStatus) -> Self {
        use e::ReviewStatus::*;
        match from {
            ReviewStatus::Archived => Archived,
            ReviewStatus::Confirmed => Confirmed,
            ReviewStatus::Created => Created,
            ReviewStatus::Rejected => Rejected,
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
impl From<e::Category> for Category {
    fn from(from: e::Category) -> Self {
        let name = from.name();
        Self {
            id: from.id.into(),
            name,
        }
    }
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

impl From<RatingContext> for e::RatingContext {
    fn from(from: RatingContext) -> Self {
        use e::RatingContext as C;
        use RatingContext as E;
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

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "extra-derive", derive(Debug, Clone))]
pub struct NewPlaceRating {
    pub entry: String,
    pub title: String,
    pub value: RatingValue,
    pub context: RatingContext,
    pub comment: String,
    pub source: Option<String>,
    pub user: Option<String>,
}

impl From<e::NewPlaceRating> for NewPlaceRating {
    fn from(from: e::NewPlaceRating) -> Self {
        let e::NewPlaceRating {
            entry,
            title,
            value,
            context,
            comment,
            source,
            user,
        } = from;
        let value = value.into();
        let context = context.into();
        NewPlaceRating {
            entry,
            title,
            value,
            context,
            comment,
            source,
            user,
        }
    }
}

impl From<NewPlaceRating> for e::NewPlaceRating {
    fn from(from: NewPlaceRating) -> Self {
        let NewPlaceRating {
            entry,
            title,
            value,
            context,
            comment,
            source,
            user,
        } = from;
        let value = value.into();
        let context = context.into();
        e::NewPlaceRating {
            entry,
            title,
            value,
            context,
            comment,
            source,
            user,
        }
    }
}

impl From<e::RatingValue> for RatingValue {
    fn from(v: e::RatingValue) -> Self {
        let v: i8 = v.into();
        RatingValue::from(v)
    }
}

impl From<RatingValue> for e::RatingValue {
    fn from(v: RatingValue) -> Self {
        e::RatingValue::from(v.0)
    }
}

impl From<e::AvgRatingValue> for AvgRatingValue {
    fn from(v: e::AvgRatingValue) -> Self {
        let v: f64 = v.into();
        AvgRatingValue::from(v)
    }
}
