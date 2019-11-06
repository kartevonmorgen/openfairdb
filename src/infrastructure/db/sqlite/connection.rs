use super::*;

use crate::core::prelude::*;
use chrono::prelude::*;
use diesel::{
    self,
    prelude::{Connection as DieselConnection, *},
    result::{DatabaseErrorKind, Error as DieselError},
};
use std::result;

type Result<T> = result::Result<T, RepoError>;

fn load_place(conn: &SqliteConnection, place_rev: models::PlaceRev) -> Result<(Place, Status)> {
    let models::PlaceRev {
        id,
        rev,
        place_uid,
        place_lic: license,
        created_at,
        created_by: created_by_id,
        status,
        title,
        description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        email,
        phone,
        homepage,
        image_url,
        image_link_url,
        ..
    } = place_rev;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
        }),
    };

    use schema::place_rev_tag::dsl as tag_dsl;
    let tags: Vec<_> = tag_dsl::place_rev_tag
        .filter(tag_dsl::place_rev_id.eq(&id))
        .load::<models::PlaceRevTag>(conn)?
        .into_iter()
        .map(|r| r.tag)
        .collect();

    let created_by = if let Some(user_id) = created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let place = Place {
        uid: place_uid.into(),
        rev: Revision::from(rev as u64),
        created: Activity {
            when: created_at.into(),
            who: created_by.map(Into::into),
        },
        license,
        title,
        description,
        location,
        contact: Some(Contact {
            email: email.map(Into::into),
            phone,
        }),
        homepage,
        image_url,
        image_link_url,
        tags,
    };

    Ok((place, status.into()))
}

fn load_place_log(
    conn: &SqliteConnection,
    place_log: models::PlaceRevStatusLog,
) -> Result<(Place, Status, ActivityLog)> {
    let models::PlaceRevStatusLog {
        id,
        rev,
        place_uid,
        place_lic: license,
        created_at,
        created_by: created_by_id,
        status,
        status_created_at,
        status_created_by: status_created_by_id,
        status_context,
        status_notes,
        title,
        description,
        lat,
        lon,
        street,
        zip,
        city,
        country,
        email,
        phone,
        homepage,
        image_url,
        image_link_url,
        ..
    } = place_log;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lon).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
        }),
    };

    use schema::place_rev_tag::dsl as tag_dsl;
    let tags: Vec<_> = tag_dsl::place_rev_tag
        .filter(tag_dsl::place_rev_id.eq(&id))
        .load::<models::PlaceRevTag>(conn)?
        .into_iter()
        .map(|r| r.tag)
        .collect();

    let created_by = if let Some(user_id) = created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let status_created_by = if status_created_by_id == created_by_id {
        created_by.clone()
    } else if let Some(user_id) = status_created_by_id {
        use schema::users::dsl;
        Some(
            schema::users::table
                .select(dsl::email)
                .filter(dsl::id.eq(&user_id))
                .first::<String>(conn)?,
        )
    } else {
        None
    };

    let place = Place {
        uid: place_uid.into(),
        rev: Revision::from(rev as u64),
        created: Activity {
            when: created_at.into(),
            who: created_by.map(Into::into),
        },
        license,
        title,
        description,
        location,
        contact: Some(Contact {
            email: email.map(Into::into),
            phone,
        }),
        homepage,
        image_url,
        image_link_url,
        tags,
    };

    let activity_log = ActivityLog {
        activity: Activity {
            when: status_created_at.into(),
            who: status_created_by.map(Into::into),
        },
        context: status_context,
        notes: status_notes,
    };

    Ok((place, status.into(), activity_log))
}

#[derive(QueryableByName)]
struct TagCountRow {
    #[sql_type = "diesel::sql_types::Text"]
    tag: String,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

fn resolve_place_id_and_rev(conn: &SqliteConnection, uid: &Uid) -> Result<(i64, Revision)> {
    use schema::place::dsl;
    Ok(schema::place::table
        .select((dsl::id, dsl::rev))
        .filter(dsl::uid.eq(uid.as_str()))
        .first::<(i64, i64)>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place '{}': {}", uid, e);
            e
        })
        .map(|(id, rev)| (id, Revision::from(rev as u64)))?)
}

fn resolve_rating_id(conn: &SqliteConnection, uid: &str) -> Result<i64> {
    use schema::place_rating::dsl;
    Ok(schema::place_rating::table
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve place rating '{}': {}", uid, e);
            e
        })?)
}

fn into_new_place_rev(
    conn: &SqliteConnection,
    place: Place,
) -> Result<(Uid, models::NewPlaceRev, Vec<String>)> {
    let Place {
        uid: place_uid,
        rev: new_revision,
        created,
        license,
        title,
        description,
        location: Location { pos, address },
        contact,
        homepage,
        image_url,
        image_link_url,
        tags,
    } = place;
    let place_id = if new_revision.is_initial() {
        // Create a new place
        let new_place = models::NewPlace {
            uid: place_uid.as_ref(),
            lic: &license,
            rev: u64::from(new_revision) as i64,
        };
        diesel::insert_into(schema::place::table)
            .values(new_place)
            .execute(conn)?;
        let (place_id, _revision) = resolve_place_id_and_rev(conn, &place_uid)?;
        debug_assert_eq!(new_revision, _revision);
        place_id
    } else {
        // Update the existing place with a new revision
        let (place_id, revision) = resolve_place_id_and_rev(conn, &place_uid)?;
        // Check for a continuous revision history without conflicts (optimistic locking)
        if revision.next() != new_revision {
            return Err(RepoError::InvalidVersion);
        }
        use schema::place::dsl as place_dsl;
        let _count = diesel::update(
            schema::place::table
                .filter(place_dsl::uid.eq(place_uid.as_str()))
                .filter(place_dsl::rev.eq(u64::from(revision) as i64)),
        )
        .set(place_dsl::rev.eq(u64::from(new_revision) as i64))
        .execute(conn)?;
        debug_assert_eq!(1, _count);
        place_id
    };
    let created_by = if let Some(ref email) = created.who {
        Some(resolve_user_id_by_email(conn, email.as_ref())?)
    } else {
        None
    };
    let Contact { email, phone } = contact.unwrap_or_default();
    let Address {
        street,
        zip,
        city,
        country,
    } = address.unwrap_or_default();
    let new_place_rev = models::NewPlaceRev {
        place_id,
        rev: u64::from(new_revision) as i64,
        created_at: created.when.into(),
        created_by,
        status: Status::created().into(),
        title,
        description,
        lat: pos.lat().to_deg(),
        lon: pos.lng().to_deg(),
        street,
        zip,
        city,
        country,
        email: email.map(Into::into),
        phone,
        homepage,
        image_url,
        image_link_url,
    };
    Ok((place_uid, new_place_rev, tags))
}

impl PlaceRepo for SqliteConnection {
    fn create_place_rev(&self, place: Place) -> Result<()> {
        let (_, new_place_rev, tags) = into_new_place_rev(self, place)?;
        diesel::insert_into(schema::place_rev::table)
            .values(&new_place_rev)
            .execute(self)?;

        use schema::place_rev::dsl;
        let place_rev_id = dsl::place_rev
            .select(dsl::id)
            .filter(dsl::place_id.eq(new_place_rev.place_id))
            .filter(dsl::rev.eq(new_place_rev.rev))
            .first::<i64>(self)
            .map_err(|e| {
                log::warn!(
                    "Newly inserted place {} revision {} not found: {}",
                    new_place_rev.place_id,
                    new_place_rev.rev,
                    e
                );
                e
            })?;

        // Insert into place_rev_activity_log
        let new_place_rev_log_status = models::NewPlaceRevStatusLog {
            place_rev_id,
            status: new_place_rev.status,
            created_at: new_place_rev.created_at,
            created_by: new_place_rev.created_by,
            context: None,
            notes: Some("created"),
        };
        diesel::insert_into(schema::place_rev_activity_log::table)
            .values(new_place_rev_log_status)
            .execute(self)?;

        // Insert into place_rev_tag
        let tags: Vec<_> = tags
            .iter()
            .map(|tag| models::NewPlaceRevTag {
                place_rev_id,
                tag: tag.as_str(),
            })
            .collect();
        diesel::insert_into(schema::place_rev_tag::table)
            .values(&tags)
            .execute(self)?;

        Ok(())
    }

    fn change_status_of_places(
        &self,
        uids: &[&str],
        status: Status,
        activity: &UserActivity,
    ) -> Result<usize> {
        use schema::place::dsl as place_dsl;
        use schema::place_rev::dsl as rev_dsl;

        let rev_ids = schema::place::table
            .inner_join(
                schema::place_rev::table.on(rev_dsl::place_id
                    .eq(place_dsl::id)
                    .and(rev_dsl::rev.eq(place_dsl::rev))),
            )
            .select(rev_dsl::id)
            .filter(place_dsl::uid.eq_any(uids))
            .load(self)?;
        let changed_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        let changed_at = activity.when.into();
        let status = status.into_inner();
        let mut total_update_count = 0;
        for rev_id in rev_ids {
            let update_count = diesel::update(
                schema::place_rev::table
                    .filter(rev_dsl::id.eq(rev_id))
                    .filter(rev_dsl::status.ne(status)),
            )
            .set(rev_dsl::status.eq(status))
            .execute(self)?;
            debug_assert!(update_count <= 1);
            if update_count > 0 {
                let new_place_rev_log_status = models::NewPlaceRevStatusLog {
                    place_rev_id: rev_id,
                    status,
                    created_at: changed_at,
                    created_by: Some(changed_by),
                    context: None,
                    notes: Some("status changed"),
                };
                diesel::insert_into(schema::place_rev_activity_log::table)
                    .values(new_place_rev_log_status)
                    .execute(self)?;
                total_update_count += update_count;
            }
        }
        Ok(total_update_count)
    }

    fn get_places(&self, place_uids: &[&str]) -> Result<Vec<(Place, Status)>> {
        use schema::place::dsl as place_dsl;
        use schema::place_rev::dsl as rev_dsl;

        let mut query = schema::place::table
            .inner_join(
                schema::place_rev::table.on(rev_dsl::place_id
                    .eq(place_dsl::id)
                    .and(rev_dsl::rev.eq(place_dsl::rev))),
            )
            .select((
                rev_dsl::id,
                rev_dsl::rev,
                rev_dsl::place_id,
                place_dsl::uid,
                place_dsl::lic,
                rev_dsl::created_at,
                rev_dsl::created_by,
                rev_dsl::status,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
            ))
            .into_boxed();
        if place_uids.is_empty() {
            warn!("Loading all entries at once");
        } else {
            // TODO: Split loading into chunks of fixed size
            info!("Loading multiple ({}) entries at once", place_uids.len());
            query = query.filter(place_dsl::uid.eq_any(place_uids));
        }

        let rows = query.load::<models::PlaceRev>(self)?;
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(load_place(self, row)?);
        }
        Ok(results)
    }

    fn get_place(&self, place_uid: &str) -> Result<(Place, Status)> {
        let places = self.get_places(&[place_uid])?;
        debug_assert!(places.len() <= 1);
        places.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn all_places(&self) -> Result<Vec<(Place, Status)>> {
        self.get_places(&[])
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, Status, ActivityLog)>> {
        use schema::place::dsl as place_dsl;
        use schema::place_rev::dsl as rev_dsl;
        use schema::place_rev_activity_log::dsl as log_dsl;

        let mut query = schema::place::table
            .inner_join(
                schema::place_rev::table.on(rev_dsl::place_id
                    .eq(place_dsl::id)
                    .and(rev_dsl::rev.eq(place_dsl::rev))),
            )
            .inner_join(
                schema::place_rev_activity_log::table.on(log_dsl::place_rev_id.eq(rev_dsl::id)),
            )
            .select((
                rev_dsl::id,
                rev_dsl::rev,
                rev_dsl::place_id,
                place_dsl::uid,
                place_dsl::lic,
                rev_dsl::created_at,
                rev_dsl::created_by,
                log_dsl::status,
                log_dsl::created_at,
                log_dsl::created_by,
                log_dsl::context,
                log_dsl::notes,
                rev_dsl::title,
                rev_dsl::description,
                rev_dsl::lat,
                rev_dsl::lon,
                rev_dsl::street,
                rev_dsl::zip,
                rev_dsl::city,
                rev_dsl::country,
                rev_dsl::email,
                rev_dsl::phone,
                rev_dsl::homepage,
                rev_dsl::image_url,
                rev_dsl::image_link_url,
            ))
            .order_by(log_dsl::created_at.desc())
            .then_order_by(log_dsl::id) // disambiguation if time stamps are equal
            .into_boxed();

        // Since (inclusive)
        if let Some(since) = params.since {
            query = query.filter(log_dsl::created_at.ge(i64::from(since)));
        }

        // Until (exclusive)
        if let Some(until) = params.until {
            query = query.filter(log_dsl::created_at.lt(i64::from(until)));
        }

        // Pagination
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            query = query.offset(offset as i64);
        }
        if let Some(limit) = pagination.limit {
            query = query.limit(limit as i64);
        }

        let rows = query.load::<models::PlaceRevStatusLog>(self)?;
        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(load_place_log(self, row)?);
        }
        Ok(results)
    }

    fn most_popular_place_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        // TODO: Diesel 1.4.x does not support the HAVING clause
        // that is required to filter the aggregated column.
        let mut sql = "SELECT tag, COUNT(*) as count \
                       FROM place_rev_tag \
                       WHERE place_rev_id IN \
                       (SELECT id FROM place_rev WHERE (place_id, rev) IN (SELECT id, rev FROM place) AND status > 0) \
                       GROUP BY tag"
            .to_string();
        if params.min_count.is_some() || params.max_count.is_some() {
            if let Some(min_count) = params.min_count {
                sql.push_str(&format!(" HAVING count>={}", min_count));
                if let Some(max_count) = params.max_count {
                    sql.push_str(&format!(" AND count<={}", max_count));
                }
            } else if let Some(max_count) = params.max_count {
                sql.push_str(&format!(" HAVING count<={}", max_count));
            }
        }
        sql.push_str(" ORDER BY count DESC, tag");
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        if let Some(limit) = pagination.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        let rows = diesel::dsl::sql_query(sql).load::<TagCountRow>(self)?;
        Ok(rows
            .into_iter()
            .map(|row| TagFrequency(row.tag, row.count as TagCount))
            .collect())
    }

    fn count_places(&self) -> Result<usize> {
        use schema::place_rev::dsl;
        Ok(schema::place_rev::table
            .select(diesel::dsl::count(dsl::place_id))
            .filter(dsl::status.ge(Status::created().into_inner()))
            .first::<i64>(self)? as usize)
    }
}

fn into_new_event_with_tags(
    conn: &SqliteConnection,
    event: Event,
) -> Result<(models::NewEvent, Vec<String>)> {
    let Event {
        uid,
        title,
        start,
        end,
        description,
        location,
        contact,
        homepage,
        created_by,
        registration,
        organizer,
        archived,
        image_url,
        image_link_url,
        tags,
        ..
    } = event;

    let mut street = None;
    let mut zip = None;
    let mut city = None;
    let mut country = None;

    let (lat, lng) = if let Some(l) = location {
        if let Some(a) = l.address {
            street = a.street;
            zip = a.zip;
            city = a.city;
            country = a.country;
        }
        (Some(l.pos.lat().to_deg()), Some(l.pos.lng().to_deg()))
    } else {
        (None, None)
    };

    let (email, telephone) = if let Some(c) = contact {
        (c.email, c.phone)
    } else {
        (None, None)
    };

    let registration = registration.map(Into::into);

    let created_by = if let Some(ref email) = created_by {
        Some(resolve_user_id_by_email(conn, email)?)
    } else {
        None
    };

    Ok((
        models::NewEvent {
            uid: uid.into(),
            title,
            description,
            start: start.timestamp(),
            end: end.map(|x| x.timestamp()),
            lat,
            lng,
            street,
            zip,
            city,
            country,
            telephone,
            email: email.map(Into::into),
            homepage,
            created_by,
            registration,
            organizer,
            archived: archived.map(Into::into),
            image_url,
            image_link_url,
        },
        tags,
    ))
}

fn resolve_event_id(conn: &SqliteConnection, uid: &str) -> Result<i64> {
    use schema::events::dsl;
    Ok(dsl::events
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first(conn)?)
}

impl EventGateway for SqliteConnection {
    fn create_event(&self, e: Event) -> Result<()> {
        let (new_event, tags) = into_new_event_with_tags(self, e)?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            // Insert event
            diesel::insert_into(schema::events::table)
                .values(&new_event)
                .execute(self)?;
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
                .map(|tag| models::NewEventTag {
                    event_id: id,
                    tag: &tag,
                })
                .collect();
            diesel::insert_or_ignore_into(schema::event_tags::table)
                .values(&tags)
                .execute(self)?;
            Ok(())
        })?;
        Ok(())
    }

    fn update_event(&self, event: &Event) -> Result<()> {
        let id = resolve_event_id(self, event.uid.as_ref())?;
        let (new_event, new_tags) = into_new_event_with_tags(self, event.clone())?;
        self.transaction::<_, diesel::result::Error, _>(|| {
            use schema::event_tags::dsl as et_dsl;
            use schema::events::dsl as e_dsl;
            // Update event
            diesel::update(e_dsl::events.filter(e_dsl::id.eq(&id)))
                .set(&new_event)
                .execute(self)?;
            // Update event tags
            let tags_diff = {
                let old_tags = et_dsl::event_tags
                    .select(et_dsl::tag)
                    .filter(et_dsl::event_id.eq(id))
                    .load(self)?;
                super::util::tags_diff(&old_tags, &new_tags)
            };
            diesel::delete(
                et_dsl::event_tags
                    .filter(et_dsl::event_id.eq(id))
                    .filter(et_dsl::tag.eq_any(&tags_diff.deleted)),
            )
            .execute(self)?;
            {
                let new_tags: Vec<_> = tags_diff
                    .added
                    .iter()
                    .map(|tag| models::NewEventTag {
                        event_id: id,
                        tag: &tag,
                    })
                    .collect();
                diesel::insert_or_ignore_into(et_dsl::event_tags)
                    .values(&new_tags)
                    .execute(self)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn get_event(&self, uid: &str) -> Result<Event> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};

        let models::EventEntity {
            id,
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
        } = e_dsl::events
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
            .filter(e_dsl::uid.eq(uid))
            .filter(e_dsl::archived.is_null())
            .first(self)?;

        let tags = et_dsl::event_tags
            .select(et_dsl::tag)
            .filter(et_dsl::event_id.eq(id))
            .load::<String>(self)?;

        let address = Address {
            street,
            zip,
            city,
            country,
        };

        let address = if address.is_empty() {
            None
        } else {
            Some(address)
        };

        let pos = if let (Some(lat), Some(lng)) = (lat, lng) {
            MapPoint::try_from_lat_lng_deg(lat, lng)
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
        let contact = if email.is_some() || telephone.is_some() {
            Some(Contact {
                email: email.map(Into::into),
                phone: telephone,
            })
        } else {
            None
        };

        let registration = registration.map(Into::into);

        Ok(Event {
            uid: uid.into(),
            title,
            start: NaiveDateTime::from_timestamp(start, 0),
            end: end.map(|x| NaiveDateTime::from_timestamp(x, 0)),
            description,
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
        })
    }

    fn all_events(&self) -> Result<Vec<Event>> {
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
            .load::<models::EventEntity>(self)?;
        let tag_rels = et_dsl::event_tags.load(self)?;
        Ok(events.into_iter().map(|e| (e, &tag_rels).into()).collect())
    }

    fn get_events(
        &self,
        start_min: Option<Timestamp>,
        start_max: Option<Timestamp>,
    ) -> Result<Vec<Event>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};
        let mut query = e_dsl::events
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
            .into_boxed();
        if let Some(start_min) = start_min {
            query = query.filter(e_dsl::start.ge(i64::from(start_min)));
        }
        if let Some(start_max) = start_max {
            query = query.filter(e_dsl::start.le(i64::from(start_max)));
        }
        let events: Vec<_> = query.load::<models::EventEntity>(self)?;
        let tag_rels = et_dsl::event_tags.load(self)?;
        Ok(events.into_iter().map(|e| (e, &tag_rels).into()).collect())
    }

    fn count_events(&self) -> Result<usize> {
        use schema::events::dsl;
        Ok(dsl::events
            .select(diesel::dsl::count(dsl::id))
            .filter(dsl::archived.is_null())
            .first::<i64>(self)? as usize)
    }

    fn archive_events(&self, uids: &[&str], archived: Timestamp) -> Result<usize> {
        use schema::events::dsl;
        let count = diesel::update(
            dsl::events
                .filter(dsl::uid.eq_any(uids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?;
        debug_assert!(count <= uids.len());
        Ok(count)
    }

    fn delete_event_with_matching_tags(&self, uid: &str, tags: &[&str]) -> Result<Option<()>> {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
        let id = resolve_event_id(self, uid)?;
        if !tags.is_empty() {
            let ids: Vec<_> = et_dsl::event_tags
                .select(et_dsl::event_id)
                .distinct()
                .filter(et_dsl::event_id.eq(id))
                .filter(et_dsl::tag.eq_any(tags))
                .load::<i64>(self)?;
            debug_assert!(ids.len() <= 1);
            if ids.is_empty() {
                return Ok(None);
            }
            debug_assert_eq!(id, *ids.first().unwrap());
        }
        diesel::delete(et_dsl::event_tags.filter(et_dsl::event_id.eq(id))).execute(self)?;
        diesel::delete(e_dsl::events.filter(e_dsl::id.eq(id))).execute(self)?;
        Ok(Some(()))
    }
}

fn resolve_user_id_by_email(conn: &SqliteConnection, email: &str) -> Result<i64> {
    use schema::users::dsl;
    Ok(dsl::users
        .select(dsl::id)
        .filter(dsl::email.eq(email))
        .first(conn)
        .map_err(|e| {
            log::warn!("Failed to resolve user by e-mail '{}': {}", email, e);
            e
        })?)
}

impl UserGateway for SqliteConnection {
    fn create_user(&self, u: &User) -> Result<()> {
        let new_user = models::NewUser::from(u);
        diesel::insert_into(schema::users::table)
            .values(&new_user)
            .execute(self)?;
        Ok(())
    }

    fn update_user(&self, u: &User) -> Result<()> {
        use schema::users::dsl;
        let new_user = models::NewUser::from(u);
        diesel::update(dsl::users.filter(dsl::email.eq(new_user.email)))
            .set(&new_user)
            .execute(self)?;
        Ok(())
    }

    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        use schema::users::dsl;
        diesel::delete(dsl::users.filter(dsl::email.eq(email))).execute(self)?;
        Ok(())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)?
            .into())
    }

    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)
            .optional()?
            .map(Into::into))
    }

    fn all_users(&self) -> Result<Vec<User>> {
        use schema::users::dsl;
        Ok(dsl::users
            .load::<models::UserEntity>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn count_users(&self) -> Result<usize> {
        use schema::users::dsl;
        Ok(dsl::users
            .select(diesel::dsl::count(dsl::id))
            .first::<i64>(self)? as usize)
    }
}

impl RatingRepository for SqliteConnection {
    fn create_rating(&self, rating: Rating) -> Result<()> {
        let Rating {
            uid,
            place_uid,
            created_at,
            archived_at,
            title,
            value,
            context,
            source,
        } = rating;
        let (place_id, _) = resolve_place_id_and_rev(self, &place_uid)?;
        let new_place_rating = models::NewPlaceRating {
            uid: uid.into(),
            place_id,
            created_at: created_at.into(),
            created_by: None,
            archived_at: archived_at.map(Into::into),
            archived_by: None,
            title,
            value: i8::from(value).into(),
            context: context.into(),
            source,
        };
        let _count = diesel::insert_into(schema::place_rating::table)
            .values(&new_place_rating)
            .execute(self)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_ratings(&self, uids: &[&str]) -> Result<Vec<Rating>> {
        use schema::place::dsl as place_dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::id,
                rating_dsl::uid,
                place_dsl::id,
                place_dsl::uid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
            ))
            .filter(rating_dsl::uid.eq_any(uids))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_rating(&self, uid: &str) -> Result<Rating> {
        let ratings = self.load_ratings(&[uid])?;
        debug_assert!(ratings.len() <= 1);
        ratings.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn load_ratings_of_entry(&self, place_uid: &str) -> Result<Vec<Rating>> {
        use schema::place::dsl as place_dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::id,
                rating_dsl::uid,
                place_dsl::id,
                place_dsl::uid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
            ))
            .filter(place_dsl::uid.eq(place_uid))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_place_uids_of_ratings(&self, uids: &[&str]) -> Result<Vec<String>> {
        use schema::place::dsl as place_dsl;
        use schema::place_rating::dsl as rating_dsl;
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select(place_dsl::uid)
            .filter(rating_dsl::uid.eq_any(uids))
            .load::<String>(self)?)
    }

    fn archive_ratings(&self, uids: &[&str], activity: &UserActivity) -> Result<usize> {
        use schema::place_rating::dsl;
        let archived_at = activity.when.into_inner();
        let archived_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        let count = diesel::update(
            schema::place_rating::table
                .filter(dsl::uid.eq_any(uids))
                .filter(dsl::archived_at.is_null()),
        )
        .set((
            dsl::archived_at.eq(Some(archived_at)),
            dsl::archived_by.eq(Some(archived_by)),
        ))
        .execute(self)?;
        debug_assert!(count <= uids.len());
        Ok(count)
    }

    fn archive_ratings_of_places(
        &self,
        place_uids: &[&str],
        activity: &UserActivity,
    ) -> Result<usize> {
        use schema::place::dsl as place_dsl;
        use schema::place_rating::dsl as rating_dsl;
        let archived_at = activity.when.into_inner();
        let archived_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        Ok(diesel::update(
            schema::place_rating::table
                .filter(
                    rating_dsl::place_id.eq_any(
                        schema::place::table
                            .select(place_dsl::id)
                            .filter(place_dsl::uid.eq_any(place_uids)),
                    ),
                )
                .filter(rating_dsl::archived_at.is_null()),
        )
        .set((
            rating_dsl::archived_at.eq(Some(archived_at)),
            rating_dsl::archived_by.eq(Some(archived_by)),
        ))
        .execute(self)?)
    }
}

impl CommentRepository for SqliteConnection {
    fn create_comment(&self, comment: Comment) -> Result<()> {
        let Comment {
            uid,
            rating_uid,
            created_at,
            archived_at,
            text,
            ..
        } = comment;
        let rating_id = resolve_rating_id(self, rating_uid.as_ref())?;
        let new_place_rating_comment = models::NewPlaceRatingComment {
            uid: uid.into(),
            rating_id,
            created_at: created_at.into(),
            created_by: None,
            archived_at: archived_at.map(Into::into),
            archived_by: None,
            text,
        };
        let _count = diesel::insert_into(schema::place_rating_comment::table)
            .values(&new_place_rating_comment)
            .execute(self)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_comments(&self, uids: &[&str]) -> Result<Vec<Comment>> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        // TODO: Split loading into chunks of fixed size
        info!("Loading multiple ({}) comments at once", uids.len());
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::id,
                comment_dsl::uid,
                comment_dsl::rating_id,
                rating_dsl::uid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::text,
            ))
            .filter(comment_dsl::uid.eq_any(uids))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_comment(&self, uid: &str) -> Result<Comment> {
        let comments = self.load_comments(&[uid])?;
        debug_assert!(comments.len() <= 1);
        comments.into_iter().next().ok_or(RepoError::NotFound)
    }

    fn load_comments_of_rating(&self, rating_uid: &str) -> Result<Vec<Comment>> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::id,
                comment_dsl::uid,
                comment_dsl::rating_id,
                rating_dsl::uid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::text,
            ))
            .filter(rating_dsl::uid.eq(rating_uid))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn archive_comments(&self, uids: &[&str], activity: &UserActivity) -> Result<usize> {
        use schema::place_rating_comment::dsl;
        let archived_at = activity.when.into_inner();
        let archived_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        let count = diesel::update(
            schema::place_rating_comment::table
                .filter(dsl::uid.eq_any(uids))
                .filter(dsl::archived_at.is_null()),
        )
        .set((
            dsl::archived_at.eq(Some(archived_at)),
            dsl::archived_by.eq(Some(archived_by)),
        ))
        .execute(self)?;
        debug_assert!(count <= uids.len());
        Ok(count)
    }

    fn archive_comments_of_ratings(
        &self,
        rating_uids: &[&str],
        activity: &UserActivity,
    ) -> Result<usize> {
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        let archived_at = activity.when.into_inner();
        let archived_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        Ok(diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::rating_id.eq_any(
                        schema::place_rating::table
                            .select(rating_dsl::id)
                            .filter(rating_dsl::uid.eq_any(rating_uids)),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(Some(archived_at)),
            comment_dsl::archived_by.eq(Some(archived_by)),
        ))
        .execute(self)?)
    }

    fn archive_comments_of_places(
        &self,
        place_uids: &[&str],
        activity: &UserActivity,
    ) -> Result<usize> {
        use schema::place::dsl as place_dsl;
        use schema::place_rating::dsl as rating_dsl;
        use schema::place_rating_comment::dsl as comment_dsl;
        let archived_at = activity.when.into_inner();
        let archived_by = resolve_user_id_by_email(self, activity.who.as_str())?;
        Ok(diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::rating_id.eq_any(
                        schema::place_rating::table.select(rating_dsl::id).filter(
                            rating_dsl::place_id.eq_any(
                                schema::place::table
                                    .select(place_dsl::id)
                                    .filter(place_dsl::uid.eq_any(place_uids)),
                            ),
                        ),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(Some(archived_at)),
            comment_dsl::archived_by.eq(Some(archived_by)),
        ))
        .execute(self)
        .optional()?
        .unwrap_or_default())
    }
}

impl Db for SqliteConnection {
    fn create_tag_if_it_does_not_exist(&self, t: &Tag) -> Result<()> {
        let res = diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self);
        if let Err(err) = res {
            match err {
                DieselError::DatabaseError(conn_err, _) => {
                    match conn_err {
                        DatabaseErrorKind::UniqueViolation => {
                            // that's ok :)
                        }
                        _ => {
                            return Err(err.into());
                        }
                    }
                }
                _ => {
                    return Err(err.into());
                }
            }
        }
        Ok(())
    }

    fn create_bbox_subscription(&self, new: &BboxSubscription) -> Result<()> {
        let user_id = resolve_user_id_by_email(self, &new.user_email)?;
        let (south_west_lat, south_west_lng) = new.bbox.south_west().to_lat_lng_deg();
        let (north_east_lat, north_east_lng) = new.bbox.north_east().to_lat_lng_deg();
        let insertable = models::NewBboxSubscription {
            uid: new.uid.as_ref(),
            user_id,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
        };
        diesel::insert_into(schema::bbox_subscriptions::table)
            .values(&insertable)
            .execute(self)?;
        Ok(())
    }

    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
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
            .load::<models::BboxSubscriptionEntity>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn all_bbox_subscriptions_by_email(&self, email: &str) -> Result<Vec<BboxSubscription>> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
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
            .load::<models::BboxSubscriptionEntity>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn delete_bbox_subscriptions_by_email(&self, email: &str) -> Result<()> {
        use schema::bbox_subscriptions::dsl as s_dsl;
        use schema::users::dsl as u_dsl;
        let users_id = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(email));
        diesel::delete(s_dsl::bbox_subscriptions.filter(s_dsl::user_id.eq_any(users_id)))
            .execute(self)?;
        Ok(())
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use schema::tags::dsl::*;
        Ok(tags
            .load::<models::Tag>(self)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn count_tags(&self) -> Result<usize> {
        use schema::tags::dsl::*;
        Ok(tags.select(diesel::dsl::count(id)).first::<i64>(self)? as usize)
    }
}

impl OrganizationGateway for SqliteConnection {
    fn create_org(&mut self, mut o: Organization) -> Result<()> {
        let org_id = o.id.clone();
        let owned_tags = std::mem::replace(&mut o.owned_tags, vec![]);
        let tag_rels: Vec<_> = owned_tags
            .iter()
            .map(|tag_id| models::StoreableOrgTagRelation {
                org_id: &org_id,
                tag_id: &tag_id,
            })
            .collect();
        let new_org = models::Organization::from(o);
        self.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(schema::organizations::table)
                .values(&new_org)
                .execute(self)?;
            diesel::insert_into(schema::org_tag_relations::table)
                //WHERE NOT EXISTS
                .values(&tag_rels)
                .execute(self)?;
            Ok(())
        })?;
        Ok(())
    }
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization> {
        use schema::{org_tag_relations::dsl as o_t_dsl, organizations::dsl as o_dsl};

        let models::Organization {
            id,
            name,
            api_token,
        } = o_dsl::organizations
            .filter(o_dsl::api_token.eq(token))
            .first(self)?;

        let owned_tags = o_t_dsl::org_tag_relations
            .filter(o_t_dsl::org_id.eq(&id))
            .load::<models::OrgTagRelation>(self)?
            .into_iter()
            .map(|r| r.tag_id)
            .collect();

        Ok(Organization {
            id,
            name,
            api_token,
            owned_tags,
        })
    }

    fn get_all_tags_owned_by_orgs(&self) -> Result<Vec<String>> {
        use schema::org_tag_relations::dsl;
        let mut tags: Vec<_> = dsl::org_tag_relations
            .load::<models::OrgTagRelation>(self)?
            .into_iter()
            .map(|r| r.tag_id)
            .collect();
        tags.dedup();
        Ok(tags)
    }
}

impl UserTokenRepo for SqliteConnection {
    fn replace_user_token(&self, token: UserToken) -> Result<EmailNonce> {
        use schema::user_tokens::dsl;
        let user_id = resolve_user_id_by_email(self, &token.email_nonce.email)?;
        let model = models::NewUserToken {
            user_id,
            nonce: token.email_nonce.nonce.to_string(),
            expires_at: token.expires_at.into(),
        };
        // Insert...
        if diesel::insert_into(schema::user_tokens::table)
            .values(&model)
            .execute(self)?
            == 0
        {
            // ...or update
            let _count = diesel::update(schema::user_tokens::table)
                .filter(dsl::user_id.eq(model.user_id))
                .set(&model)
                .execute(self)?;
            debug_assert_eq!(1, _count);
        }
        Ok(token.email_nonce)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        use schema::user_tokens::dsl as t_dsl;
        use schema::users::dsl as u_dsl;
        let token = self.get_user_token_by_email(&email_nonce.email)?;
        let user_id_subselect = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(&email_nonce.email));
        let target = t_dsl::user_tokens
            .filter(t_dsl::nonce.eq(email_nonce.nonce.to_string()))
            .filter(t_dsl::user_id.eq_any(user_id_subselect));
        if diesel::delete(target).execute(self)? == 0 {
            return Err(RepoError::NotFound);
        }
        debug_assert_eq!(email_nonce, &token.email_nonce);
        Ok(token)
    }

    fn discard_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        use schema::user_tokens::dsl;
        Ok(diesel::delete(
            dsl::user_tokens.filter(dsl::expires_at.lt::<i64>(expired_before.into())),
        )
        .execute(self)?)
    }

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken> {
        use schema::user_tokens::dsl as t_dsl;
        use schema::users::dsl as u_dsl;
        Ok(t_dsl::user_tokens
            .inner_join(u_dsl::users)
            .select((u_dsl::id, t_dsl::nonce, t_dsl::expires_at, u_dsl::email))
            .filter(u_dsl::email.eq(email))
            .first::<models::UserTokenEntity>(self)?
            .into())
    }
}
