use super::*;
use ofdb_entities as e;
use thiserror::Error;

impl From<e::time::Timestamp> for UnixTimeMillis {
    fn from(from: e::time::Timestamp) -> Self {
        Self(from.as_millis())
    }
}

impl From<e::time::Timestamp> for UnixTimeSeconds {
    fn from(from: e::time::Timestamp) -> Self {
        Self(from.as_secs())
    }
}

impl TryFrom<UnixTimeSeconds> for e::time::Timestamp {
    type Error = e::time::OutOfRangeError;
    fn try_from(from: UnixTimeSeconds) -> Result<Self, Self::Error> {
        Self::try_from_secs(from.0)
    }
}

impl TryFrom<UnixTimeMillis> for e::time::Timestamp {
    type Error = e::time::OutOfRangeError;
    fn try_from(from: UnixTimeMillis) -> Result<Self, Self::Error> {
        Self::try_from_millis(from.0)
    }
}

impl From<e::links::CustomLink> for CustomLink {
    fn from(from: e::links::CustomLink) -> Self {
        let e::links::CustomLink {
            url,
            title,
            description,
        } = from;
        Self {
            url: url.to_string(),
            title,
            description,
        }
    }
}

// TODO: use TryFrom
impl From<CustomLink> for e::links::CustomLink {
    fn from(from: CustomLink) -> Self {
        let CustomLink {
            url,
            title,
            description,
        } = from;
        Self {
            url: url.parse().unwrap(),
            title,
            description,
        }
    }
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
            email: email.to_string(),
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
            name: organizer,
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

        let start = start.into();
        let end = end.map(e::time::Timestamp::from).map(Into::into);

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
            email: email.map(e::email::EmailAddress::into_string),
            telephone,
            homepage: homepage.map(Into::into),
            tags,
            registration,
            organizer,
            image_url: image_url.map(Into::into),
            image_link_url: image_link_url.map(Into::into),
        }
    }
}

impl From<e::clearance::PendingClearanceForPlace> for PendingClearanceForPlace {
    fn from(from: e::clearance::PendingClearanceForPlace) -> Self {
        let e::clearance::PendingClearanceForPlace {
            place_id,
            created_at,
            last_cleared_revision,
        } = from;
        Self {
            place_id: place_id.into(),
            created_at: created_at.into(),
            last_cleared_revision: last_cleared_revision.map(Into::into),
        }
    }
}

impl From<ClearanceForPlace> for e::clearance::ClearanceForPlace {
    fn from(from: ClearanceForPlace) -> Self {
        let ClearanceForPlace {
            place_id,
            cleared_revision,
        } = from;
        Self {
            place_id: place_id.into(),
            cleared_revision: cleared_revision.map(Into::into),
        }
    }
}

impl From<e::geo::MapPoint> for LatLonDegrees {
    fn from(from: e::geo::MapPoint) -> Self {
        Self(from.lat().to_deg(), from.lng().to_deg())
    }
}

impl From<e::geo::MapPoint> for MapPoint {
    fn from(from: e::geo::MapPoint) -> Self {
        Self {
            lat: from.lat().to_deg(),
            lng: from.lng().to_deg(),
        }
    }
}

impl TryFrom<LatLonDegrees> for e::geo::MapPoint {
    type Error = e::geo::CoordRangeError;

    fn try_from(from: LatLonDegrees) -> Result<Self, Self::Error> {
        e::geo::MapPoint::try_from_lat_lng_deg(from.0, from.1)
    }
}

impl From<e::geo::MapBbox> for MapBbox {
    fn from(bbox: e::geo::MapBbox) -> Self {
        Self {
            sw: MapPoint::from(bbox.southwest()),
            ne: MapPoint::from(bbox.northeast()),
        }
    }
}

impl From<e::address::Address> for Address {
    fn from(from: e::address::Address) -> Self {
        let e::address::Address {
            street,
            zip,
            city,
            country,
            state,
        } = from;
        Self {
            street,
            zip,
            city,
            country,
            state,
        }
    }
}

impl From<Address> for e::address::Address {
    fn from(from: Address) -> Self {
        let Address {
            street,
            zip,
            city,
            country,
            state,
        } = from;
        Self {
            street,
            zip,
            city,
            country,
            state,
        }
    }
}

impl From<e::location::Location> for Location {
    fn from(from: e::location::Location) -> Self {
        let e::location::Location { pos, address } = from;
        Self {
            latlon: pos.into(),
            address: address.map(Into::into).unwrap_or_default(),
        }
    }
}

// TODO: use TryFrom here
impl From<Location> for e::location::Location {
    fn from(from: Location) -> Self {
        let Location { latlon, address } = from;
        Self {
            pos: latlon.try_into().unwrap(),
            address: Some(address.into()),
        }
    }
}

impl From<e::contact::Contact> for Contact {
    fn from(from: e::contact::Contact) -> Self {
        let e::contact::Contact { name, email, phone } = from;
        Self {
            name,
            email: email.map(e::email::EmailAddress::into_string),
            phone,
        }
    }
}

impl From<e::links::Links> for Links {
    fn from(from: e::links::Links) -> Self {
        let e::links::Links {
            homepage,
            image,
            image_href,
            custom,
        } = from;
        Self {
            homepage: homepage.map(Into::into),
            image: image.map(Into::into),
            image_href: image_href.map(Into::into),
            custom: custom.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Links> for e::links::Links {
    fn from(from: Links) -> Self {
        let Links {
            homepage,
            image,
            image_href,
            custom,
        } = from;
        Self {
            homepage: homepage.and_then(|url| url.parse().ok()),
            image: image.and_then(|url| url.parse().ok()),
            image_href: image_href.and_then(|url| url.parse().ok()),
            custom: custom.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ContactConversionError {
    #[error(transparent)]
    Email(#[from] e::email::EmailAddressParseError),
}

impl TryFrom<Contact> for e::contact::Contact {
    type Error = ContactConversionError;
    fn try_from(from: Contact) -> Result<Self, Self::Error> {
        let Contact { name, email, phone } = from;
        let email = email.map(|email| email.parse()).transpose()?;
        Ok(Self { name, email, phone })
    }
}

impl From<e::activity::Activity> for Activity {
    fn from(from: e::activity::Activity) -> Self {
        let e::activity::Activity { at, by } = from;
        Self {
            at: at.into(),
            by: by.map(e::email::EmailAddress::into_string),
        }
    }
}

#[derive(Debug, Error)]
pub enum ActivityConversionError {
    #[error(transparent)]
    Email(#[from] e::email::EmailAddressParseError),
    #[error(transparent)]
    Time(#[from] e::time::OutOfRangeError),
}

impl TryFrom<Activity> for e::activity::Activity {
    type Error = ActivityConversionError;
    fn try_from(from: Activity) -> Result<Self, Self::Error> {
        let Activity { at, by } = from;
        let by = by.map(|email| email.parse()).transpose()?;
        let at = at.try_into()?;
        Ok(Self { at, by })
    }
}

impl From<e::place::PlaceRoot> for PlaceRoot {
    fn from(from: e::place::PlaceRoot) -> Self {
        let e::place::PlaceRoot { id, license } = from;
        Self {
            id: id.into(),
            license,
        }
    }
}

impl From<PlaceRoot> for e::place::PlaceRoot {
    fn from(from: PlaceRoot) -> Self {
        let PlaceRoot { id, license } = from;
        Self {
            id: id.into(),
            license,
        }
    }
}

impl From<e::place::PlaceRevision> for PlaceRevision {
    fn from(from: e::place::PlaceRevision) -> Self {
        let e::place::PlaceRevision {
            revision,
            created,
            title,
            description,
            location,
            contact,
            opening_hours,
            founded_on,
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
            opening_hours: opening_hours.map(Into::into),
            founded_on: founded_on.map(Into::into),
            links: links.map(Into::into).unwrap_or_default(),
            tags,
        }
    }
}

#[derive(Debug, Error)]
pub enum PlaceRevisionConversionError {
    #[error(transparent)]
    Contact(#[from] ContactConversionError),
    #[error(transparent)]
    Activity(#[from] ActivityConversionError),
    #[error(transparent)]
    Email(#[from] e::email::EmailAddressParseError),
}

impl TryFrom<PlaceRevision> for e::place::PlaceRevision {
    type Error = PlaceRevisionConversionError;
    fn try_from(from: PlaceRevision) -> Result<Self, Self::Error> {
        let PlaceRevision {
            revision,
            created,
            title,
            description,
            location,
            contact,
            opening_hours,
            founded_on,
            links,
            tags,
        } = from;

        Ok(Self {
            revision: revision.into(),
            created: created.try_into()?,
            title,
            description,
            location: location.into(),
            contact: Some(contact.try_into()?),
            opening_hours: opening_hours.map(Into::into),
            founded_on: founded_on.map(Into::into),
            links: Some(links.into()),
            tags,
        })
    }
}

impl From<e::place::PlaceHistory> for PlaceHistory {
    fn from(from: e::place::PlaceHistory) -> Self {
        let e::place::PlaceHistory { place, revisions } = from;
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

#[derive(Debug, Error)]
pub enum PlaceHistoryConversionError {
    #[error(transparent)]
    PlaceRevision(#[from] PlaceRevisionConversionError),
    #[error(transparent)]
    ReviewStatusLog(#[from] ReviewStatusLogConversionError),
}

impl TryFrom<PlaceHistory> for e::place::PlaceHistory {
    type Error = PlaceHistoryConversionError;
    fn try_from(from: PlaceHistory) -> Result<Self, Self::Error> {
        let PlaceHistory { place, revisions } = from;
        let place = place.into();
        let revisions = revisions
            .into_iter()
            .map(|(place_revision, reviews)| {
                place_revision
                    .try_into()
                    .map_err(Self::Error::PlaceRevision)
                    .and_then(|place_revision| {
                        reviews
                            .into_iter()
                            .map(TryInto::try_into)
                            .collect::<Result<_, _>>()
                            .map_err(Self::Error::ReviewStatusLog)
                            .map(|reviews| (place_revision, reviews))
                    })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { place, revisions })
    }
}

impl From<e::activity::ActivityLog> for ActivityLog {
    fn from(from: e::activity::ActivityLog) -> Self {
        let e::activity::ActivityLog {
            activity: e::activity::Activity { at, by },
            context: ctx,
            comment,
        } = from;
        Self {
            at: at.into(),
            by: by.map(e::email::EmailAddress::into_string),
            ctx,
            comment,
        }
    }
}

#[derive(Debug, Error)]
pub enum ActivityLogConversionError {
    #[error(transparent)]
    Activity(#[from] ActivityConversionError),
}

impl TryFrom<ActivityLog> for e::activity::ActivityLog {
    type Error = ActivityLogConversionError;
    fn try_from(from: ActivityLog) -> Result<Self, Self::Error> {
        let ActivityLog {
            at,
            by,
            ctx: context,
            comment,
        } = from;
        let activity = Activity { at, by }.try_into()?;
        Ok(Self {
            activity,
            context,
            comment,
        })
    }
}

impl From<e::review::ReviewStatusLog> for ReviewStatusLog {
    fn from(from: e::review::ReviewStatusLog) -> Self {
        let e::review::ReviewStatusLog {
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

#[derive(Debug, Error)]
pub enum ReviewStatusLogConversionError {
    #[error(transparent)]
    ActivityLog(#[from] ActivityLogConversionError),
}

impl TryFrom<ReviewStatusLog> for e::review::ReviewStatusLog {
    type Error = ReviewStatusLogConversionError;
    fn try_from(from: ReviewStatusLog) -> Result<Self, Self::Error> {
        let ReviewStatusLog { rev, act, status } = from;
        Ok(Self {
            revision: rev.into(),
            activity: act.try_into()?,
            status: status.into(),
        })
    }
}
