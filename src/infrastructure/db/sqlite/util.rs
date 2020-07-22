use super::models::*;
use crate::core::{
    entities as e,
    prelude::{ParameterError, Result},
    util::{
        geo::{MapBbox, MapPoint},
        nonce::Nonce,
        time::Timestamp,
    },
};
use chrono::prelude::*;
use url::Url;

pub(crate) fn load_url(url: String) -> Option<Url> {
    match url.parse() {
        Ok(url) => Some(url),
        Err(err) => {
            // The database should only contain valid URLs
            log::error!("Failed to load URL '{}' from database: {}", url, err);
            None
        }
    }
}

pub(crate) fn registration_type_from_i16(i: i16) -> e::RegistrationType {
    use crate::core::entities::RegistrationType::*;
    match i {
        1 => Email,
        2 => Phone,
        3 => Homepage,
        _ => {
            error!(
                "Convertion Error:
                       Invalid registration type:
                       {} should be one of 1,2,3;
                       Use 'Phone' instead.",
                i
            );
            Phone
        }
    }
}

pub(crate) fn registration_type_into_i16(x: e::RegistrationType) -> i16 {
    use crate::core::entities::RegistrationType::*;
    match x {
        Email => 1,
        Phone => 2,
        Homepage => 3,
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn registration_type_from_i16() {
        use crate::core::entities::RegistrationType::*;
        assert_eq!(super::registration_type_from_i16(1), Email);
        assert_eq!(super::registration_type_from_i16(2), Phone);
        assert_eq!(super::registration_type_from_i16(3), Homepage);
        assert_eq!(super::registration_type_from_i16(7), Phone);
    }

    #[test]
    fn registration_type_into_i16() {
        use crate::core::entities::RegistrationType::*;
        let e: i16 = super::registration_type_into_i16(Email);
        let p: i16 = super::registration_type_into_i16(Phone);
        let u: i16 = super::registration_type_into_i16(Homepage);
        assert_eq!(e, 1);
        assert_eq!(p, 2);
        assert_eq!(u, 3);
    }
}

pub(crate) fn event_from_event_entity_and_tags(e: EventEntity, tag_rels: &[EventTag]) -> e::Event {
    let EventEntity {
        id,
        uid,
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
        email,
        telephone,
        homepage,
        registration,
        organizer,
        archived,
        image_url,
        image_link_url,
        created_by_email,
        ..
    } = e;
    let tags = tag_rels
        .iter()
        .filter(|r| r.event_id == id)
        .map(|r| &r.tag)
        .cloned()
        .collect();
    let address = if street.is_some()
        || zip.is_some()
        || city.is_some()
        || country.is_some()
        || state.is_some()
    {
        Some(e::Address {
            street,
            zip,
            city,
            country,
            state,
        })
    } else {
        None
    };
    let pos = if let (Some(lat), Some(lng)) = (lat, lng) {
        MapPoint::try_from_lat_lng_deg(lat, lng)
    } else {
        None
    };
    let location = if address.is_some() || lat.is_some() || lng.is_some() {
        Some(e::Location {
            pos: pos.unwrap_or_default(),
            address,
        })
    } else {
        None
    };
    let contact = if email.is_some() || telephone.is_some() {
        Some(e::Contact {
            email: email.map(Into::into),
            phone: telephone,
        })
    } else {
        None
    };

    let registration = registration.map(registration_type_from_i16);

    e::Event {
        id: uid.into(),
        title,
        description,
        start: NaiveDateTime::from_timestamp(start, 0),
        end: end.map(|x| NaiveDateTime::from_timestamp(x, 0)),
        location,
        contact,
        homepage: homepage.and_then(load_url),
        tags,
        created_by: created_by_email,
        registration,
        organizer,
        archived: archived.map(Timestamp::from_inner),
        image_url: image_url.and_then(load_url),
        image_link_url: image_link_url.and_then(load_url),
    }
}

impl From<Tag> for e::Tag {
    fn from(t: Tag) -> e::Tag {
        e::Tag { id: t.id }
    }
}

impl From<e::Tag> for Tag {
    fn from(t: e::Tag) -> Tag {
        Tag { id: t.id }
    }
}

impl<'a> From<&'a e::User> for NewUser<'a> {
    fn from(u: &'a e::User) -> NewUser<'a> {
        use num_traits::ToPrimitive;
        Self {
            email: &u.email,
            email_confirmed: u.email_confirmed,
            password: u.password.to_string(),
            role: u.role.to_i16().unwrap_or_else(|| {
                warn!("Could not convert role {:?} to i16. Use 0 instead.", u.role);
                0
            }),
        }
    }
}

impl From<UserEntity> for e::User {
    fn from(u: UserEntity) -> e::User {
        use num_traits::FromPrimitive;
        let UserEntity {
            email,
            email_confirmed,
            password,
            role,
            ..
        } = u;
        Self {
            email,
            email_confirmed,
            password: password.into(),
            role: e::Role::from_i16(role).unwrap_or_else(|| {
                warn!(
                    "Could not cast role from i16 (value: {}). Use {:?} instead.",
                    role,
                    e::Role::default()
                );
                e::Role::default()
            }),
        }
    }
}

impl From<PlaceRatingComment> for e::Comment {
    fn from(c: PlaceRatingComment) -> Self {
        let PlaceRatingComment {
            id,
            rating_id,
            created_at,
            archived_at,
            text,
            ..
        } = c;
        Self {
            id: id.into(),
            rating_id: rating_id.into(),
            created_at: Timestamp::from_inner(created_at),
            archived_at: archived_at.map(Timestamp::from_inner),
            text,
        }
    }
}

impl From<PlaceRating> for e::Rating {
    fn from(r: PlaceRating) -> Self {
        let PlaceRating {
            id,
            place_id,
            created_at,
            archived_at,
            title,
            context,
            value,
            source,
            ..
        } = r;
        Self {
            id: id.into(),
            place_id: place_id.into(),
            created_at: Timestamp::from_inner(created_at),
            archived_at: archived_at.map(Timestamp::from_inner),
            title,
            value: (value as i8).into(),
            context: rating_context_from_str(&context).unwrap(),
            source,
        }
    }
}

impl From<BboxSubscriptionEntity> for e::BboxSubscription {
    fn from(from: BboxSubscriptionEntity) -> Self {
        let BboxSubscriptionEntity {
            uid,
            user_email,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
            ..
        } = from;
        let south_west =
            MapPoint::try_from_lat_lng_deg(south_west_lat, south_west_lng).unwrap_or_default();
        let north_east =
            MapPoint::try_from_lat_lng_deg(north_east_lat, north_east_lng).unwrap_or_default();
        let bbox = MapBbox::new(south_west, north_east);
        Self {
            id: uid.into(),
            user_email,
            bbox,
        }
    }
}

impl From<UserTokenEntity> for e::UserToken {
    fn from(from: UserTokenEntity) -> Self {
        Self {
            email_nonce: e::EmailNonce {
                email: from.user_email,
                nonce: from.nonce.parse::<Nonce>().unwrap_or_default(),
            },
            expires_at: Timestamp::from_inner(from.expires_at),
        }
    }
}

pub(crate) fn rating_context_to_string(context: e::RatingContext) -> String {
    match context {
        e::RatingContext::Diversity => "diversity",
        e::RatingContext::Renewable => "renewable",
        e::RatingContext::Fairness => "fairness",
        e::RatingContext::Humanity => "humanity",
        e::RatingContext::Transparency => "transparency",
        e::RatingContext::Solidarity => "solidarity",
    }
    .into()
}

fn rating_context_from_str(context: &str) -> Result<e::RatingContext> {
    Ok(match context {
        "diversity" => e::RatingContext::Diversity,
        "renewable" => e::RatingContext::Renewable,
        "fairness" => e::RatingContext::Fairness,
        "humanity" => e::RatingContext::Humanity,
        "transparency" => e::RatingContext::Transparency,
        "solidarity" => e::RatingContext::Solidarity,
        _ => {
            return Err(ParameterError::RatingContext(context.into()).into());
        }
    })
}

impl From<e::Organization> for NewOrganization {
    fn from(o: e::Organization) -> Self {
        let e::Organization {
            id,
            name,
            api_token,
            moderated_tags: _,
        } = o;
        NewOrganization {
            id: id.into(),
            name,
            api_token,
        }
    }
}

impl From<OrganizationTag> for e::ModeratedTag {
    fn from(from: OrganizationTag) -> Self {
        let OrganizationTag {
            org_rowid: _,
            tag_label,
            tag_moderation_flags,
        } = from;
        let label = tag_label;
        // TODO: Verify that value is valid
        let moderation_flags = (tag_moderation_flags as e::TagModerationFlagsValue).into();
        Self {
            label,
            moderation_flags,
        }
    }
}

impl From<OrganizationTagWithId> for (e::Id, e::ModeratedTag) {
    fn from(from: OrganizationTagWithId) -> Self {
        let OrganizationTagWithId {
            org_id,
            tag_label,
            tag_moderation_flags,
        } = from;
        let label = tag_label;
        // TODO: Verify that value is valid
        let moderation_flags = (tag_moderation_flags as e::TagModerationFlagsValue).into();
        (
            org_id.into(),
            e::ModeratedTag {
                label,
                moderation_flags,
            },
        )
    }
}

pub struct ChangeSet<T> {
    pub added: Vec<T>,
    pub deleted: Vec<T>,
}

pub fn tags_diff(old: &[String], new: &[String]) -> ChangeSet<String> {
    let mut added = vec![];
    let mut deleted = vec![];

    for t in new {
        if !old.iter().any(|x| x == t) {
            added.push(t.to_owned());
        }
    }

    for t in old {
        if !new.iter().any(|x| x == t) {
            deleted.push(t.to_owned());
        }
    }

    ChangeSet { added, deleted }
}

impl From<PendingAuthorizationForPlace> for e::PendingAuthorizationForPlace {
    fn from(from: PendingAuthorizationForPlace) -> Self {
        let PendingAuthorizationForPlace {
            place_id,
            created_at,
            last_authorized_revision,
        } = from;
        let last_authorized_revision =
            last_authorized_revision.map(|rev| e::Revision::from(rev as u64));
        Self {
            place_id: place_id.into(),
            created_at: e::TimestampMs::from_inner(created_at),
            last_authorized_revision,
        }
    }
}

#[test]
fn test_tag_diff() {
    let x = tags_diff(&[], &["b".into()]);
    assert_eq!(x.added, vec!["b"]);
    assert!(x.deleted.is_empty());

    let x = tags_diff(&["a".into()], &[]);
    assert!(x.added.is_empty());
    assert_eq!(x.deleted, vec!["a"]);

    let x = tags_diff(&["a".into()], &["b".into()]);
    assert_eq!(x.added, vec!["b"]);
    assert_eq!(x.deleted, vec!["a"]);

    let x = tags_diff(&["a".into(), "b".into()], &["b".into()]);
    assert!(x.added.is_empty());
    assert_eq!(x.deleted, vec!["a"]);
}
