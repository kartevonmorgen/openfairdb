// NOTE:
// All timestamps with the `_at` postfix are stored
// as unix timestamp in **milli**seconds.
//
// TODO: Create a new type for milliseconds and seconds.

use anyhow::anyhow;
use diesel::{
    self,
    prelude::{Connection as DieselConnection, *},
    result::{DatabaseErrorKind, Error as DieselError},
};

use ofdb_core::{
    entities::*,
    repositories::{self as repo, *},
    util::{geo::MapPoint, time::Timestamp},
};

use super::{util::load_url, *};

mod comment;
mod event;
mod org;
mod place;
mod place_clearance;
mod rating;
mod reminder;
mod subscription;
mod tag;
mod user;
mod user_token;

type Result<T> = std::result::Result<T, repo::Error>;

pub fn from_diesel_err(err: DieselError) -> repo::Error {
    match err {
        DieselError::NotFound => repo::Error::NotFound,
        _ => repo::Error::Other(err.into()),
    }
}

fn load_email_by_user_id(conn: &mut SqliteConnection, user_id: i64) -> Result<Option<String>> {
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
    conn: &mut SqliteConnection,
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
    conn: &mut SqliteConnection,
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
    conn: &mut SqliteConnection,
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
    conn: &mut SqliteConnection,
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

fn resolve_organization_rowid(conn: &mut SqliteConnection, id: &Id) -> Result<i64> {
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

fn resolve_place_rowid(conn: &mut SqliteConnection, id: &Id) -> Result<i64> {
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
    conn: &mut SqliteConnection,
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
    conn: &mut SqliteConnection,
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

fn resolve_rating_rowid(conn: &mut SqliteConnection, id: &str) -> Result<i64> {
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
    conn: &mut SqliteConnection,
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

fn into_new_event_with_tags(
    conn: &mut SqliteConnection,
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

fn resolve_user_created_by_email(conn: &mut SqliteConnection, email: &str) -> Result<i64> {
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

impl<'a> CategoryRepo for DbReadOnly<'a> {}
