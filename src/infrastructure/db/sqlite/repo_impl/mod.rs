// NOTE:
// All timestamps with the `_at` postfix are stored
// as unix timestamp in **milli**seconds.
//
// TODO: Create a new type for milliseconds and seconds.

use std::{fmt::Write as _, result};

use anyhow::anyhow;
use diesel::{
    self,
    prelude::{Connection as DieselConnection, *},
    result::{DatabaseErrorKind, Error as DieselError},
};

use super::{util::load_url, *};
use crate::core::prelude::*;
use ofdb_core::repositories as repo;

mod comment;
mod org;
mod place;
mod place_clearance;
mod rating;
mod user_token;

type Result<T> = result::Result<T, repo::Error>;

pub fn from_diesel_err(err: DieselError) -> repo::Error {
    match err {
        DieselError::NotFound => repo::Error::NotFound,
        _ => repo::Error::Other(err.into()),
    }
}

fn load_email_by_user_id(conn: &SqliteConnection, user_id: i64) -> Result<Option<String>> {
    use schema::users::dsl;
    let email = schema::users::table
        .select(dsl::email)
        .filter(dsl::id.eq(&user_id))
        .first::<String>(conn)
        .optional()
        .map_err(from_diesel_err)?;
    if email.is_none() {
        // This should never happen
        log::warn!(
            "Referential integrity violation: User with id = {} not found",
            user_id
        );
    }
    Ok(email)
}

fn load_review_status(status: ReviewStatusPrimitive) -> Result<ReviewStatus> {
    ReviewStatus::try_from(status)
        .ok_or_else(|| anyhow!("Invalid review status: {}", status).into())
}

fn load_place_revision_tags(
    conn: &SqliteConnection,
    place_revision_rowid: i64,
) -> Result<Vec<String>> {
    use schema::place_revision_tag::dsl;
    Ok(schema::place_revision_tag::table
        .filter(dsl::parent_rowid.eq(&place_revision_rowid))
        .load::<models::PlaceRevisionTag>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(
            |models::PlaceRevisionTag {
                 parent_rowid: _,
                 tag,
             }| tag,
        )
        .collect())
}

fn load_place_revision_custom_links(
    conn: &SqliteConnection,
    place_revision_rowid: i64,
) -> Result<Vec<CustomLink>> {
    use schema::place_revision_custom_link::dsl;
    Ok(schema::place_revision_custom_link::table
        .filter(dsl::parent_rowid.eq(&place_revision_rowid))
        .load::<models::PlaceRevisionCustomLink>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .filter_map(
            |models::PlaceRevisionCustomLink {
                 parent_rowid: _,
                 url,
                 title,
                 description,
             }| {
                url.parse()
                    .map_err(|err| {
                        // This should never happen if URLs have been validated properly on insert
                        log::error!("Failed to load custom link with invalid URL: {}", err);
                        err
                    })
                    .ok()
                    .map(|url| CustomLink {
                        url,
                        title,
                        description,
                    })
            },
        )
        .collect())
}

fn load_place(
    conn: &SqliteConnection,
    place: models::JoinedPlaceRevision,
) -> Result<(Place, ReviewStatus)> {
    let models::JoinedPlaceRevision {
        id,
        place_id,
        place_license: license,
        rev,
        created_at,
        created_by: created_by_id,
        current_status,
        title,
        desc: description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email,
        phone,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        ..
    } = place;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
            state,
        }),
    };

    let tags = load_place_revision_tags(conn, id)?;

    let custom_links = load_place_revision_custom_links(conn, id)?;

    let created_by = created_by_id
        .map(|user_id| load_email_by_user_id(conn, user_id))
        .transpose()?
        .flatten();

    let founded_on = founded_on.map(util::parse_date).transpose()?;

    let place = Place {
        id: place_id.into(),
        license,
        revision: Revision::from(rev as u64),
        created: Activity {
            at: Timestamp::from_millis(created_at),
            by: created_by.map(Into::into),
        },
        title,
        description,
        location,
        contact: Some(Contact {
            name: contact_name,
            email: email.map(Into::into),
            phone,
        }),
        links: Some(Links {
            homepage: homepage.and_then(load_url),
            image: image_url.and_then(load_url),
            image_href: image_link_url.and_then(load_url),
            custom: custom_links,
        }),
        opening_hours: opening_hours.map(Into::into),
        founded_on,
        tags,
    };

    Ok((place, load_review_status(current_status)?))
}

fn load_place_with_status_review(
    conn: &SqliteConnection,
    place_with_status_review: models::JoinedPlaceRevisionWithStatusReview,
) -> Result<(Place, ReviewStatus, ActivityLog)> {
    let models::JoinedPlaceRevisionWithStatusReview {
        id,
        rev,
        created_at,
        created_by: created_by_id,
        title,
        desc: description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email,
        phone,
        homepage,
        opening_hours,
        founded_on,
        image_url,
        image_link_url,
        place_id,
        place_license: license,
        review_created_at,
        review_created_by: review_created_by_id,
        review_status,
        review_context,
        review_comment,
        review_rev: _,
    } = place_with_status_review;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
            state,
        }),
    };

    let tags = load_place_revision_tags(conn, id)?;

    let custom_links = load_place_revision_custom_links(conn, id)?;

    let created_by = created_by_id
        .map(|user_id| load_email_by_user_id(conn, user_id))
        .transpose()?
        .flatten();

    let links = Links {
        homepage: homepage.and_then(load_url),
        image: image_url.and_then(load_url),
        image_href: image_link_url.and_then(load_url),
        custom: custom_links,
    };

    let contact = Contact {
        name: contact_name,
        email: email.map(Into::into),
        phone,
    };

    let review_created_by = if review_created_by_id == created_by_id {
        created_by.clone()
    } else if let Some(user_id) = review_created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)
                .map_err(from_diesel_err)?,
        )
    } else {
        None
    };

    let founded_on = founded_on.map(util::parse_date).transpose()?;

    let place = Place {
        id: place_id.into(),
        license,
        revision: Revision::from(rev as u64),
        created: Activity {
            at: Timestamp::from_millis(created_at),
            by: created_by.map(Into::into),
        },
        title,
        description,
        location,
        contact: Some(contact),
        opening_hours: opening_hours.map(Into::into),
        founded_on,
        links: Some(links),
        tags,
    };

    let activity_log = ActivityLog {
        activity: Activity {
            at: Timestamp::from_millis(review_created_at),
            by: review_created_by.map(Into::into),
        },
        context: review_context,
        comment: review_comment,
    };

    Ok((place, load_review_status(review_status)?, activity_log))
}

#[derive(QueryableByName)]
struct TagCountRow {
    #[sql_type = "diesel::sql_types::Text"]
    tag: String,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

fn resolve_organization_rowid(conn: &SqliteConnection, id: &Id) -> Result<i64> {
    use schema::organization::dsl;
    schema::organization::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve organization id '{}': {}", id, e);
            e
        })
        .map_err(from_diesel_err)
}

fn resolve_place_rowid(conn: &SqliteConnection, id: &Id) -> Result<i64> {
    use schema::place::dsl;
    schema::place::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place id '{}': {}", id, e);
            e
        })
        .map_err(from_diesel_err)
}

fn resolve_place_rowid_verify_revision(
    conn: &SqliteConnection,
    id: &Id,
    revision: Revision,
) -> Result<i64> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};
    let revision = RevisionValue::from(revision);
    schema::place::table
        .inner_join(schema::place_revision::table)
        .select(dsl::rowid)
        .filter(dsl::id.eq(id.as_str()))
        .filter(rev_dsl::rev.eq(revision as i64))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!(
                "Failed to resolve place id '{}' with revision {}: {}",
                id,
                revision,
                e
            );
            e
        })
        .map_err(from_diesel_err)
}

fn resolve_place_rowid_with_current_revision(
    conn: &SqliteConnection,
    id: &Id,
) -> Result<(i64, Revision)> {
    use schema::place::dsl;
    schema::place::table
        .select((dsl::rowid, dsl::current_rev))
        .filter(dsl::id.eq(id.as_str()))
        .first::<(i64, i64)>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place id '{}': {}", id, e);
            e
        })
        .map_err(from_diesel_err)
        .map(|(id, rev)| (id, Revision::from(rev as u64)))
}

fn resolve_rating_rowid(conn: &SqliteConnection, id: &str) -> Result<i64> {
    use schema::place_rating::dsl;
    schema::place_rating::table
        .select(dsl::rowid)
        .filter(dsl::id.eq(id))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place rating id '{}': {}", id, e);
            e
        })
        .map_err(from_diesel_err)
}

fn into_new_place_revision(
    conn: &SqliteConnection,
    place: Place,
) -> Result<(Id, models::NewPlaceRevision, Vec<String>, Vec<CustomLink>)> {
    let Place {
        id: place_id,
        license,
        revision: new_revision,
        created,
        title,
        description,
        location: Location { pos, address },
        contact,
        opening_hours,
        founded_on,
        tags,
        links,
    } = place;
    let parent_rowid = if new_revision.is_initial() {
        // Create a new place
        let new_place = models::NewPlace {
            id: place_id.as_ref(),
            license: &license,
            current_rev: u64::from(new_revision) as i64,
        };
        diesel::insert_into(schema::place::table)
            .values(new_place)
            .execute(conn)
            .map_err(from_diesel_err)?;
        let (rowid, _revision) = resolve_place_rowid_with_current_revision(conn, &place_id)?;
        debug_assert_eq!(new_revision, _revision);
        rowid
    } else {
        // Update the existing place with a new revision
        let (rowid, revision) = resolve_place_rowid_with_current_revision(conn, &place_id)?;
        // Check for a contiguous revision history without conflicts (optimistic
        // locking)
        if revision.next() != new_revision {
            return Err(repo::Error::InvalidVersion);
        }
        use schema::place::dsl;
        let _count = diesel::update(
            schema::place::table
                .filter(dsl::rowid.eq(rowid))
                .filter(dsl::current_rev.eq(u64::from(revision) as i64)),
        )
        .set(dsl::current_rev.eq(u64::from(new_revision) as i64))
        .execute(conn)
        .map_err(from_diesel_err)?;
        debug_assert_eq!(1, _count);
        rowid
    };
    let created_by = if let Some(ref email) = created.by {
        Some(resolve_user_created_by_email(conn, email.as_ref())?)
    } else {
        None
    };
    let Contact {
        name: contact_name,
        email,
        phone,
    } = contact.unwrap_or_default();
    debug_assert!(pos.is_valid());
    let Address {
        street,
        zip,
        city,
        country,
        state,
    } = address.unwrap_or_default();
    let Links {
        homepage,
        image: image_url,
        image_href: image_link_url,
        custom: custom_links,
    } = links.unwrap_or_default();
    let new_place = models::NewPlaceRevision {
        parent_rowid,
        rev: u64::from(new_revision) as i64,
        created_at: created.at.as_millis(),
        created_by,
        current_status: ReviewStatus::Created.into(),
        title,
        description,
        lat: pos.lat().to_deg(),
        lon: pos.lng().to_deg(),
        street,
        zip,
        city,
        country,
        state,
        contact_name,
        email: email.map(Into::into),
        phone,
        homepage: homepage.map(Into::into),
        opening_hours: opening_hours.map(Into::into),
        founded_on: founded_on.map(util::to_date_string),
        image_url: image_url.map(Into::into),
        image_link_url: image_link_url.map(Into::into),
    };
    Ok((place_id, new_place, tags, custom_links))
}

pub struct Connection<'c>(&'c SqliteConnection);

impl<'c> Connection<'c> {
    pub const fn new(conn: &'c SqliteConnection) -> Self {
        Self(conn)
    }
}

impl<'c> Deref for Connection<'c> {
    type Target = SqliteConnection;
    fn deref(&self) -> &'c Self::Target {
        self.0
    }
}

fn into_new_event_with_tags(
    conn: &SqliteConnection,
    event: Event,
) -> Result<(models::NewEvent, Vec<String>)> {
    let Event {
        id,
        title,
        start,
        end,
        description,
        location,
        contact,
        homepage,
        created_by,
        registration,
        archived,
        image_url,
        image_link_url,
        tags,
        ..
    } = event;

    let (lat, lng, address) = if let Some(l) = location {
        let Location { pos, address } = l;
        // The position might be invalid if no geo coords have
        // been provided. Nevertheless some address fields might
        // have been provided.
        if pos.is_valid() {
            (
                Some(pos.lat().to_deg()),
                Some(pos.lng().to_deg()),
                address.unwrap_or_default(),
            )
        } else {
            (None, None, address.unwrap_or_default())
        }
    } else {
        (None, None, Default::default())
    };
    let Address {
        street,
        zip,
        city,
        country,
        state,
    } = address;

    let (organizer, email, telephone) = if let Some(c) = contact {
        (c.name, c.email, c.phone)
    } else {
        (None, None, None)
    };

    let registration = registration.map(util::registration_type_into_i16);

    let created_by = if let Some(ref email) = created_by {
        Some(resolve_user_created_by_email(conn, email)?)
    } else {
        None
    };

    Ok((
        models::NewEvent {
            uid: id.into(),
            title,
            description,
            start: start.as_secs(),
            end: end.map(Timestamp::as_secs),
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            telephone,
            email: email.map(Into::into),
            homepage: homepage.map(Into::into),
            created_by,
            registration,
            organizer,
            archived: archived.map(Timestamp::as_secs),
            image_url: image_url.map(Into::into),
            image_link_url: image_link_url.map(Into::into),
        },
        tags,
    ))
}

fn resolve_event_id(conn: &SqliteConnection, uid: &str) -> Result<i64> {
    use schema::events::dsl;
    dsl::events
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first(conn)
        .map_err(from_diesel_err)
}

impl EventGateway for Connection<'_> {
    fn create_event(&self, e: Event) -> Result<()> {
        let (new_event, tags) = into_new_event_with_tags(self, e)?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            // Insert event
            diesel::insert_into(schema::events::table)
                .values(&new_event)
                .execute(self.deref())?;
            let id = resolve_event_id(self, new_event.uid.as_ref()).map_err(|err| {
                warn!(
                    "Failed to resolve id of newly created event {}: {}",
                    new_event.uid, err,
                );
                diesel::result::Error::RollbackTransaction
            })?;
            // Insert event tags
            let tags: Vec<_> = tags
                .iter()
                .map(|tag| models::NewEventTag { event_id: id, tag })
                .collect();
            diesel::insert_or_ignore_into(schema::event_tags::table)
                .values(&tags)
                .execute(self.deref())?;
            Ok(())
        })
        .map_err(from_diesel_err)?;
        Ok(())
    }

    fn update_event(&self, event: &Event) -> Result<()> {
        let id = resolve_event_id(self, event.id.as_ref())?;
        let (new_event, new_tags) = into_new_event_with_tags(self, event.clone())?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
            // Update event
            diesel::update(e_dsl::events.filter(e_dsl::id.eq(&id)))
                .set(&new_event)
                .execute(self.deref())?;
            // Update event tags
            let tags_diff = {
                let old_tags = et_dsl::event_tags
                    .select(et_dsl::tag)
                    .filter(et_dsl::event_id.eq(id))
                    .load(self.deref())?;
                super::util::tags_diff(&old_tags, &new_tags)
            };
            diesel::delete(
                et_dsl::event_tags
                    .filter(et_dsl::event_id.eq(id))
                    .filter(et_dsl::tag.eq_any(&tags_diff.deleted)),
            )
            .execute(self.deref())?;
            {
                let new_tags: Vec<_> = tags_diff
                    .added
                    .iter()
                    .map(|tag| models::NewEventTag { event_id: id, tag })
                    .collect();
                diesel::insert_or_ignore_into(et_dsl::event_tags)
                    .values(&new_tags)
                    .execute(self.deref())?;
            }
            Ok(())
        })
        .map_err(from_diesel_err)?;
        Ok(())
    }

    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};

        let rows = e_dsl::events
            .left_outer_join(u_dsl::users)
            .select((
                e_dsl::id,
                e_dsl::uid,
                e_dsl::title,
                e_dsl::description,
                e_dsl::start,
                e_dsl::end,
                e_dsl::lat,
                e_dsl::lng,
                e_dsl::street,
                e_dsl::zip,
                e_dsl::city,
                e_dsl::country,
                e_dsl::state,
                e_dsl::email,
                e_dsl::telephone,
                e_dsl::homepage,
                e_dsl::created_by,
                e_dsl::registration,
                e_dsl::organizer,
                e_dsl::archived,
                e_dsl::image_url,
                e_dsl::image_link_url,
                u_dsl::email.nullable(),
            ))
            .filter(e_dsl::uid.eq_any(ids))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::start)
            .load::<models::EventEntity>(self.deref())
            .map_err(from_diesel_err)?;
        debug_assert!(rows.len() <= ids.len());
        let mut events = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            let models::EventEntity {
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
            } = row;

            let tags = et_dsl::event_tags
                .select(et_dsl::tag)
                .filter(et_dsl::event_id.eq(id))
                .load::<String>(self.deref())
                .map_err(from_diesel_err)?;

            let address = Address {
                street,
                zip,
                city,
                country,
                state,
            };

            let address = if address.is_empty() {
                None
            } else {
                Some(address)
            };

            let pos = if let (Some(lat), Some(lng)) = (lat, lng) {
                MapPoint::try_from_lat_lng_deg(lat, lng)
                    .map(Some)
                    .unwrap_or_default()
            } else {
                None
            };
            let location = if pos.is_some() || address.is_some() {
                Some(Location {
                    pos: pos.unwrap_or_default(),
                    address,
                })
            } else {
                None
            };
            let contact = if organizer.is_some() || email.is_some() || telephone.is_some() {
                Some(Contact {
                    name: organizer,
                    email: email.map(Into::into),
                    phone: telephone,
                })
            } else {
                None
            };

            let registration = registration.map(util::registration_type_from_i16);

            let event = Event {
                id: uid.into(),
                title,
                start: Timestamp::from_secs(start),
                end: end.map(Timestamp::from_secs),
                description,
                location,
                contact,
                homepage: homepage.and_then(load_url),
                tags,
                created_by: created_by_email,
                registration,
                archived: archived.map(Timestamp::from_secs),
                image_url: image_url.and_then(load_url),
                image_link_url: image_link_url.and_then(load_url),
            };
            events.push(event);
        }

        Ok(events)
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        let events = self.get_events_chronologically(&[id])?;
        debug_assert!(events.len() <= 1);
        events.into_iter().next().ok_or(repo::Error::NotFound)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};
        let events: Vec<_> = e_dsl::events
            .left_outer_join(u_dsl::users)
            .select((
                e_dsl::id,
                e_dsl::uid,
                e_dsl::title,
                e_dsl::description,
                e_dsl::start,
                e_dsl::end,
                e_dsl::lat,
                e_dsl::lng,
                e_dsl::street,
                e_dsl::zip,
                e_dsl::city,
                e_dsl::country,
                e_dsl::state,
                e_dsl::email,
                e_dsl::telephone,
                e_dsl::homepage,
                e_dsl::created_by,
                e_dsl::registration,
                e_dsl::organizer,
                e_dsl::archived,
                e_dsl::image_url,
                e_dsl::image_link_url,
                u_dsl::email.nullable(),
            ))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::start)
            .load::<models::EventEntity>(self.deref())
            .map_err(from_diesel_err)?;
        let tag_rels = et_dsl::event_tags
            .load(self.deref())
            .map_err(from_diesel_err)?;
        Ok(events
            .into_iter()
            .map(|e| util::event_from_event_entity_and_tags(e, &tag_rels))
            .collect())
    }

    fn count_events(&self) -> Result<usize> {
        use schema::events::dsl;
        Ok(dsl::events
            .select(diesel::dsl::count(dsl::id))
            .filter(dsl::archived.is_null())
            .first::<i64>(self.deref())
            .map_err(from_diesel_err)? as usize)
    }

    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        use schema::events::dsl;
        let count = diesel::update(
            dsl::events
                .filter(dsl::uid.eq_any(ids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(archived.as_secs())))
        .execute(self.deref())
        .map_err(from_diesel_err)?;
        debug_assert!(count <= ids.len());
        Ok(count)
    }

    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
        let id = resolve_event_id(self, id)?;
        if !tags.is_empty() {
            let ids: Vec<_> = et_dsl::event_tags
                .select(et_dsl::event_id)
                .distinct()
                .filter(et_dsl::event_id.eq(id))
                .filter(et_dsl::tag.eq_any(tags))
                .load::<i64>(self.deref())
                .map_err(from_diesel_err)?;
            debug_assert!(ids.len() <= 1);
            if ids.is_empty() {
                return Ok(false);
            }
            debug_assert_eq!(id, *ids.first().unwrap());
        }
        diesel::delete(et_dsl::event_tags.filter(et_dsl::event_id.eq(id)))
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        diesel::delete(e_dsl::events.filter(e_dsl::id.eq(id)))
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(true)
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        use schema::{event_tags, events, organization_tag};
        Ok(events::table
            .select(events::id)
            .filter(events::uid.eq(id))
            .filter(
                events::id.eq_any(
                    event_tags::table.select(event_tags::event_id).filter(
                        event_tags::tag
                            .eq_any(organization_tag::table.select(organization_tag::tag_label)),
                    ),
                ),
            )
            .first::<i64>(self.deref())
            .optional()
            .map_err(from_diesel_err)?
            .is_some())
    }
}

fn resolve_user_created_by_email(conn: &SqliteConnection, email: &str) -> Result<i64> {
    use schema::users::dsl;
    dsl::users
        .select(dsl::id)
        .filter(dsl::email.eq(email))
        .first(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve user by e-mail '{}': {}", email, e);
            e
        })
        .map_err(from_diesel_err)
}

impl UserGateway for Connection<'_> {
    fn create_user(&self, u: &User) -> Result<()> {
        let new_user = models::NewUser::from(u);
        diesel::insert_into(schema::users::table)
            .values(&new_user)
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(())
    }

    fn update_user(&self, u: &User) -> Result<()> {
        use schema::users::dsl;
        let new_user = models::NewUser::from(u);
        diesel::update(dsl::users.filter(dsl::email.eq(new_user.email)))
            .set(&new_user)
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(())
    }

    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        use schema::users::dsl;
        diesel::delete(dsl::users.filter(dsl::email.eq(email)))
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self.deref())
            .map_err(from_diesel_err)?
            .into())
    }

    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self.deref())
            .optional()
            .map_err(from_diesel_err)?
            .map(Into::into))
    }

    fn all_users(&self) -> Result<Vec<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .load::<models::UserEntity>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn count_users(&self) -> Result<usize> {
        use schema::users::dsl;
        Ok(dsl::users
            .select(diesel::dsl::count(dsl::id))
            .first::<i64>(self.deref())
            .map_err(from_diesel_err)? as usize)
    }
}

impl Db for Connection<'_> {
    fn create_tag_if_it_does_not_exist(&self, t: &Tag) -> Result<()> {
        let res = diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self.deref());
        if let Err(err) = res {
            match err {
                DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                    // that's ok :)
                }
                _ => {
                    return Err(from_diesel_err(err));
                }
            }
        }
        Ok(())
    }

    fn create_bbox_subscription(&self, new: &BboxSubscription) -> Result<()> {
        let user_id = resolve_user_created_by_email(self, &new.user_email)?;
        let (south_west_lat, south_west_lng) = new.bbox.southwest().to_lat_lng_deg();
        let (north_east_lat, north_east_lng) = new.bbox.northeast().to_lat_lng_deg();
        let insertable = models::NewBboxSubscription {
            uid: new.id.as_ref(),
            user_id,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
        };
        diesel::insert_into(schema::bbox_subscriptions::table)
            .values(&insertable)
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(())
    }

    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
        Ok(s_dsl::bbox_subscriptions
            .inner_join(u_dsl::users)
            .select((
                s_dsl::id,
                s_dsl::uid,
                s_dsl::user_id,
                s_dsl::south_west_lat,
                s_dsl::south_west_lng,
                s_dsl::north_east_lat,
                s_dsl::north_east_lng,
                u_dsl::email,
            ))
            .load::<models::BboxSubscriptionEntity>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn all_bbox_subscriptions_by_email(&self, email: &str) -> Result<Vec<BboxSubscription>> {
        use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
        Ok(s_dsl::bbox_subscriptions
            .inner_join(u_dsl::users)
            .filter(u_dsl::email.eq(email))
            .select((
                s_dsl::id,
                s_dsl::uid,
                s_dsl::user_id,
                s_dsl::south_west_lat,
                s_dsl::south_west_lng,
                s_dsl::north_east_lat,
                s_dsl::north_east_lng,
                u_dsl::email,
            ))
            .load::<models::BboxSubscriptionEntity>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn delete_bbox_subscriptions_by_email(&self, email: &str) -> Result<()> {
        use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
        let users_id = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(email));
        diesel::delete(s_dsl::bbox_subscriptions.filter(s_dsl::user_id.eq_any(users_id)))
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        Ok(())
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use schema::tags::dsl::*;
        Ok(tags
            .load::<models::Tag>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn count_tags(&self) -> Result<usize> {
        use schema::tags::dsl::*;
        Ok(tags
            .select(diesel::dsl::count(id))
            .first::<i64>(self.deref())
            .map_err(from_diesel_err)? as usize)
    }
}
