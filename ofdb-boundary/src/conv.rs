use super::*;
use e::url::Url;
use ofdb_entities as e;
use std::convert::{TryFrom, TryInto};

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

impl From<e::clearance::PendingClearanceForPlace> for PendingClearanceForPlace {
    fn from(from: e::clearance::PendingClearanceForPlace) -> Self {
        let e::clearance::PendingClearanceForPlace {
            place_id,
            created_at,
            last_cleared_revision,
        } = from;
        Self {
            place_id: place_id.into(),
            created_at: created_at.into_inner(),
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
            email: email.map(Into::into),
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
            homepage: homepage.map(Url::into_string),
            image: image.map(Url::into_string),
            image_href: image_href.map(Url::into_string),
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

impl From<Contact> for e::contact::Contact {
    fn from(from: Contact) -> Self {
        let Contact { name, email, phone } = from;
        Self {
            name,
            email: email.map(Into::into),
            phone,
        }
    }
}

impl From<e::activity::Activity> for Activity {
    fn from(from: e::activity::Activity) -> Self {
        let e::activity::Activity { at, by } = from;
        Self {
            at: at.into_inner(),
            by: by.map(Into::into),
        }
    }
}

impl From<Activity> for e::activity::Activity {
    fn from(from: Activity) -> Self {
        let Activity { at, by } = from;
        Self {
            at: e::time::TimestampMs::from_inner(at),
            by: by.map(Into::into),
        }
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

impl From<PlaceRevision> for e::place::PlaceRevision {
    fn from(from: PlaceRevision) -> Self {
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
        Self {
            revision: revision.into(),
            created: created.into(),
            title,
            description,
            location: location.into(),
            contact: Some(contact.into()),
            opening_hours: opening_hours.map(Into::into),
            founded_on: founded_on.map(Into::into),
            links: Some(links.into()),
            tags,
        }
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

impl From<PlaceHistory> for e::place::PlaceHistory {
    fn from(from: PlaceHistory) -> Self {
        let PlaceHistory { place, revisions } = from;
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

impl From<e::activity::ActivityLog> for ActivityLog {
    fn from(from: e::activity::ActivityLog) -> Self {
        let e::activity::ActivityLog {
            activity: e::activity::Activity { at, by },
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

impl From<ActivityLog> for e::activity::ActivityLog {
    fn from(from: ActivityLog) -> Self {
        let ActivityLog {
            at,
            by,
            ctx: context,
            comment,
        } = from;
        let at = e::time::TimestampMs::from_inner(at);
        let activity = e::activity::Activity {
            at,
            by: by.map(Into::into),
        };
        Self {
            activity,
            context,
            comment,
        }
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

impl From<ReviewStatusLog> for e::review::ReviewStatusLog {
    fn from(from: ReviewStatusLog) -> Self {
        let ReviewStatusLog { rev, act, status } = from;
        Self {
            revision: rev.into(),
            activity: act.into(),
            status: status.into(),
        }
    }
}
