use super::*;

impl EventRepo for DbReadOnly<'_> {
    fn create_event(&self, _ev: Event) -> Result<()> {
        unreachable!();
    }
    fn update_event(&self, _ev: &Event) -> Result<()> {
        unreachable!();
    }
    fn archive_events(&self, _ids: &[&str], _archived: Timestamp) -> Result<usize> {
        unreachable!();
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        get_event(&mut self.conn.borrow_mut(), id)
    }
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        get_events_chronologically(&mut self.conn.borrow_mut(), ids)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        all_events_chronologically(&mut self.conn.borrow_mut())
    }

    fn count_events(&self) -> Result<usize> {
        count_events(&mut self.conn.borrow_mut())
    }

    fn delete_event_with_matching_tags(&self, _id: &str, _tags: &[&str]) -> Result<bool> {
        unreachable!();
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        is_event_owned_by_any_organization(&mut self.conn.borrow_mut(), id)
    }
}

impl EventRepo for DbReadWrite<'_> {
    fn create_event(&self, ev: Event) -> Result<()> {
        create_event(&mut self.conn.borrow_mut(), ev)
    }
    fn update_event(&self, ev: &Event) -> Result<()> {
        update_event(&mut self.conn.borrow_mut(), ev)
    }
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        archive_events(&mut self.conn.borrow_mut(), ids, archived)
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        get_event(&mut self.conn.borrow_mut(), id)
    }
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        get_events_chronologically(&mut self.conn.borrow_mut(), ids)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        all_events_chronologically(&mut self.conn.borrow_mut())
    }

    fn count_events(&self) -> Result<usize> {
        count_events(&mut self.conn.borrow_mut())
    }

    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool> {
        delete_event_with_matching_tags(&mut self.conn.borrow_mut(), id, tags)
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        is_event_owned_by_any_organization(&mut self.conn.borrow_mut(), id)
    }
}

impl EventRepo for DbConnection<'_> {
    fn create_event(&self, ev: Event) -> Result<()> {
        create_event(&mut self.conn.borrow_mut(), ev)
    }
    fn update_event(&self, ev: &Event) -> Result<()> {
        update_event(&mut self.conn.borrow_mut(), ev)
    }
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        archive_events(&mut self.conn.borrow_mut(), ids, archived)
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        get_event(&mut self.conn.borrow_mut(), id)
    }
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        get_events_chronologically(&mut self.conn.borrow_mut(), ids)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        all_events_chronologically(&mut self.conn.borrow_mut())
    }

    fn count_events(&self) -> Result<usize> {
        count_events(&mut self.conn.borrow_mut())
    }

    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool> {
        delete_event_with_matching_tags(&mut self.conn.borrow_mut(), id, tags)
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        is_event_owned_by_any_organization(&mut self.conn.borrow_mut(), id)
    }
}

fn create_event(conn: &mut SqliteConnection, e: Event) -> Result<()> {
    let (new_event, tags) = into_new_event_with_tags(conn, e)?;
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Insert event
        diesel::insert_into(schema::events::table)
            .values(&new_event)
            .execute(conn)?;
        let id = resolve_event_id(conn, new_event.uid.as_ref()).map_err(|err| {
            log::warn!(
                "Failed to resolve id of newly created event {}: {}",
                new_event.uid,
                err,
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
            .execute(conn)?;
        Ok(())
    })
    .map_err(from_diesel_err)?;
    Ok(())
}

fn update_event(conn: &mut SqliteConnection, event: &Event) -> Result<()> {
    let id = resolve_event_id(conn, event.id.as_ref())?;
    let (new_event, new_tags) = into_new_event_with_tags(conn, event.clone())?;
    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
        // Update event
        diesel::update(e_dsl::events.filter(e_dsl::id.eq(&id)))
            .set(&new_event)
            .execute(conn)?;
        // Update event tags
        let tags_diff = {
            let old_tags = et_dsl::event_tags
                .select(et_dsl::tag)
                .filter(et_dsl::event_id.eq(id))
                .load(conn)?;
            super::util::tags_diff(&old_tags, &new_tags)
        };
        diesel::delete(
            et_dsl::event_tags
                .filter(et_dsl::event_id.eq(id))
                .filter(et_dsl::tag.eq_any(&tags_diff.deleted)),
        )
        .execute(conn)?;
        {
            let new_tags: Vec<_> = tags_diff
                .added
                .iter()
                .map(|tag| models::NewEventTag { event_id: id, tag })
                .collect();
            diesel::insert_or_ignore_into(et_dsl::event_tags)
                .values(&new_tags)
                .execute(conn)?;
        }
        Ok(())
    })
    .map_err(from_diesel_err)?;
    Ok(())
}

fn get_events_chronologically(conn: &mut SqliteConnection, ids: &[&str]) -> Result<Vec<Event>> {
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
        .load::<models::EventEntity>(conn)
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
            .load::<String>(conn)
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
                email: email.map(EmailAddress::new_unchecked),
                phone: telephone,
            })
        } else {
            None
        };

        let registration = registration.map(util::registration_type_from_i16);
        let created_by = created_by_email.map(EmailAddress::new_unchecked);

        let event = Event {
            id: uid.into(),
            title,
            start: Timestamp::try_from_secs(start).unwrap(),
            end: end.map(Timestamp::try_from_secs).transpose().unwrap(),
            description,
            location,
            contact,
            homepage: homepage.and_then(load_url),
            tags,
            created_by,
            registration,
            archived: archived.map(Timestamp::try_from_secs).transpose().unwrap(),
            image_url: image_url.and_then(load_url),
            image_link_url: image_link_url.and_then(load_url),
        };
        events.push(event);
    }

    Ok(events)
}

fn get_event(conn: &mut SqliteConnection, id: &str) -> Result<Event> {
    let events = get_events_chronologically(conn, &[id])?;
    debug_assert!(events.len() <= 1);
    events.into_iter().next().ok_or(repo::Error::NotFound)
}

fn all_events_chronologically(conn: &mut SqliteConnection) -> Result<Vec<Event>> {
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
        .load::<models::EventEntity>(conn)
        .map_err(from_diesel_err)?;
    let tag_rels = et_dsl::event_tags.load(conn).map_err(from_diesel_err)?;
    Ok(events
        .into_iter()
        .map(|e| util::event_from_event_entity_and_tags(e, &tag_rels))
        .collect())
}

fn count_events(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::events::dsl;
    Ok(dsl::events
        .select(diesel::dsl::count(dsl::id))
        .filter(dsl::archived.is_null())
        .first::<i64>(conn)
        .map_err(from_diesel_err)? as usize)
}

fn archive_events(conn: &mut SqliteConnection, ids: &[&str], archived: Timestamp) -> Result<usize> {
    use schema::events::dsl;
    let count = diesel::update(
        dsl::events
            .filter(dsl::uid.eq_any(ids))
            .filter(dsl::archived.is_null()),
    )
    .set(dsl::archived.eq(Some(archived.as_secs())))
    .execute(conn)
    .map_err(from_diesel_err)?;
    debug_assert!(count <= ids.len());
    Ok(count)
}

fn delete_event_with_matching_tags(
    conn: &mut SqliteConnection,
    id: &str,
    tags: &[&str],
) -> Result<bool> {
    use schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
    let id = resolve_event_id(conn, id)?;
    if !tags.is_empty() {
        let ids: Vec<_> = et_dsl::event_tags
            .select(et_dsl::event_id)
            .distinct()
            .filter(et_dsl::event_id.eq(id))
            .filter(et_dsl::tag.eq_any(tags))
            .load::<i64>(conn)
            .map_err(from_diesel_err)?;
        debug_assert!(ids.len() <= 1);
        if ids.is_empty() {
            return Ok(false);
        }
        debug_assert_eq!(id, *ids.first().unwrap());
    }
    diesel::delete(et_dsl::event_tags.filter(et_dsl::event_id.eq(id)))
        .execute(conn)
        .map_err(from_diesel_err)?;
    diesel::delete(e_dsl::events.filter(e_dsl::id.eq(id)))
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(true)
}

fn is_event_owned_by_any_organization(conn: &mut SqliteConnection, id: &str) -> Result<bool> {
    use schema::{event_tags, events, organization_tag};
    Ok(events::table
        .select(events::id)
        .filter(events::uid.eq(id))
        .filter(
            events::id.eq_any(event_tags::table.select(event_tags::event_id).filter(
                event_tags::tag.eq_any(organization_tag::table.select(organization_tag::tag_label)),
            )),
        )
        .first::<i64>(conn)
        .optional()
        .map_err(from_diesel_err)?
        .is_some())
}

fn resolve_event_id(conn: &mut SqliteConnection, uid: &str) -> Result<i64> {
    use schema::events::dsl;
    dsl::events
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first(conn)
        .map_err(from_diesel_err)
}
