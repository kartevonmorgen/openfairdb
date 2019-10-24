use super::models::*;
use crate::core::{
    entities as e,
    prelude::{Error, ParameterError, Result},
    util::{
        geo::{MapBbox, MapPoint},
        nonce::Nonce,
    },
};
use chrono::prelude::*;
use std::str::FromStr;

impl From<e::Entry> for Entry {
    fn from(e: e::Entry) -> Self {
        let e::Entry {
            id,
            osm_node,
            created,
            archived,
            version,
            title,
            description,
            location,
            contact,
            homepage,
            license,
            image_url,
            image_link_url,
            ..
        } = e;

        let e::Location { pos, address } = location;
        debug_assert!(pos.is_valid());

        let e::Address {
            street,
            zip,
            city,
            country,
        } = address.unwrap_or_else(|| e::Address {
            street: None,
            zip: None,
            city: None,
            country: None,
        });

        let e::Contact { email, telephone } = contact.unwrap_or_else(|| e::Contact {
            email: None,
            telephone: None,
        });

        Entry {
            id,
            osm_node: osm_node.map(|x| x as i64),
            created: created.into(),
            archived: archived.map(Into::into),
            version: version as i64,
            current: true,
            title,
            description,
            lat: pos.lat().to_deg(),
            lng: pos.lng().to_deg(),
            street,
            zip,
            city,
            country,
            email,
            telephone,
            homepage,
            license,
            image_url,
            image_link_url,
        }
    }
}

impl From<i16> for e::RegistrationType {
    fn from(i: i16) -> Self {
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
}

#[test]
fn registration_type_from_i16() {
    use crate::core::entities::RegistrationType::{self, *};
    assert_eq!(RegistrationType::from(1), Email);
    assert_eq!(RegistrationType::from(2), Phone);
    assert_eq!(RegistrationType::from(3), Homepage);
    assert_eq!(RegistrationType::from(7), Phone);
}

impl Into<i16> for e::RegistrationType {
    fn into(self) -> i16 {
        use crate::core::entities::RegistrationType::*;
        match self {
            Email => 1,
            Phone => 2,
            Homepage => 3,
        }
    }
}

#[test]
fn registration_type_into_i16() {
    use crate::core::entities::RegistrationType::*;
    let e: i16 = Email.into();
    let p: i16 = Phone.into();
    let u: i16 = Homepage.into();
    assert_eq!(e, 1);
    assert_eq!(p, 2);
    assert_eq!(u, 3);
}

impl From<(Entry, Vec<String>, Vec<String>)> for e::Entry {
    fn from(d: (Entry, Vec<String>, Vec<String>)) -> Self {
        let (e, categories, tags) = d;
        let Entry {
            id,
            version,
            created,
            archived,
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
            license,
            homepage,
            image_url,
            image_link_url,
            ..
        } = e;
        let location = e::Location {
            pos: MapPoint::try_from_lat_lng_deg(lat, lng).unwrap_or_default(),
            address: if street.is_some() || zip.is_some() || city.is_some() || country.is_some() {
                Some(e::Address {
                    street,
                    zip,
                    city,
                    country,
                })
            } else {
                None
            },
        };
        let contact = if email.is_some() || telephone.is_some() {
            Some(e::Contact { email, telephone })
        } else {
            None
        };
        e::Entry {
            id,
            osm_node: e.osm_node.map(|x| x as u64),
            created: created.into(),
            archived: archived.map(Into::into),
            version: version as u64,
            title,
            description,
            location,
            contact,
            homepage,
            categories,
            tags,
            license,
            image_url,
            image_link_url,
        }
    }
}

impl From<(EventEntity, &Vec<EventTag>)> for e::Event {
    fn from(d: (EventEntity, &Vec<EventTag>)) -> Self {
        let (e, tag_rels) = d;
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
        let address = if street.is_some() || zip.is_some() || city.is_some() || country.is_some() {
            Some(e::Address {
                street,
                zip,
                city,
                country,
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
            Some(e::Contact { email, telephone })
        } else {
            None
        };

        let registration = registration.map(Into::into);

        e::Event {
            uid: uid.into(),
            title,
            description,
            start: NaiveDateTime::from_timestamp(start, 0),
            end: end.map(|x| NaiveDateTime::from_timestamp(x, 0)),
            location,
            contact,
            homepage,
            tags,
            created_by: created_by_email,
            registration,
            organizer,
            archived: archived.map(Into::into),
            image_url,
            image_link_url,
        }
    }
}

impl From<Category> for e::Category {
    fn from(c: Category) -> e::Category {
        let Category {
            id,
            name,
            created,
            version,
        } = c;
        e::Category {
            id,
            name,
            created,
            version: version as u64,
        }
    }
}

impl From<e::Category> for Category {
    fn from(c: e::Category) -> Category {
        let e::Category {
            id,
            name,
            created,
            version,
        } = c;
        Category {
            id,
            name,
            created,
            version: version as i64,
        }
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

impl From<Comment> for e::Comment {
    fn from(c: Comment) -> e::Comment {
        let Comment {
            id,
            created,
            archived,
            text,
            rating_id,
        } = c;
        e::Comment {
            id,
            created: created.into(),
            archived: archived.map(Into::into),
            text,
            rating_id,
        }
    }
}

impl From<e::Comment> for Comment {
    fn from(c: e::Comment) -> Comment {
        let e::Comment {
            id,
            created,
            archived,
            text,
            rating_id,
        } = c;
        Comment {
            id,
            created: created.into(),
            archived: archived.map(Into::into),
            text,
            rating_id,
        }
    }
}

impl From<Rating> for e::Rating {
    fn from(r: Rating) -> e::Rating {
        let Rating {
            id,
            entry_id,
            created,
            archived,
            title,
            context,
            value,
            source,
        } = r;
        e::Rating {
            id,
            entry_id,
            created: created.into(),
            archived: archived.map(Into::into),
            title,
            value: (value as i8).into(),
            context: context.parse().unwrap(),
            source,
        }
    }
}

impl From<e::Rating> for Rating {
    fn from(r: e::Rating) -> Rating {
        let e::Rating {
            id,
            created,
            archived,
            title,
            context,
            value,
            source,
            entry_id,
        } = r;
        Rating {
            id,
            created: created.into(),
            archived: archived.map(Into::into),
            title,
            value: i8::from(value).into(),
            context: context.into(),
            source,
            entry_id,
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
            uid: uid.into(),
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
            expires_at: from.expires_at.into(),
        }
    }
}

impl From<e::RatingContext> for String {
    fn from(context: e::RatingContext) -> String {
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
}

impl FromStr for e::RatingContext {
    type Err = Error;
    fn from_str(context: &str) -> Result<e::RatingContext> {
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
}

impl From<e::Organization> for Organization {
    fn from(o: e::Organization) -> Self {
        let e::Organization {
            id,
            name,
            api_token,
            ..
        } = o;
        Organization {
            id,
            name,
            api_token,
        }
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
