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

fn reset_current_entry_before_insert_new_version(conn: &SqliteConnection, id: &str) -> Result<()> {
    use self::schema::entries::dsl;
    let count = diesel::update(
        dsl::entries
            .filter(dsl::id.eq(id))
            .filter(dsl::current.eq(true))
            .filter(dsl::archived.is_null()),
    )
    .set(dsl::current.eq(false))
    .execute(conn)?;
    match count {
        // Either none or one entry version was marked as
        // `current` before and has been reset.
        n if n <= 1 => Ok(()),
        // Otherwise refuse to insert a new entry version.
        n if n > 1 => Err(RepoError::TooManyFound),
        _ => unreachable!(),
    }
}

fn load_entry(conn: &SqliteConnection, entry: models::Entry) -> Result<Entry> {
    use self::schema::entry_category_relations::dsl as ec_dsl;
    use self::schema::entry_tag_relations::dsl as et_dsl;

    let models::Entry {
        id,
        osm_node,
        created,
        archived,
        version,
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
        homepage,
        license,
        image_url,
        image_link_url,
        ..
    } = entry;

    let location = Location {
        pos: MapPoint::try_from_lat_lng_deg(lat, lng).unwrap_or_default(),
        address: Some(Address {
            street,
            zip,
            city,
            country,
        }),
    };

    let categories = ec_dsl::entry_category_relations
        .filter(ec_dsl::entry_id.eq(&id))
        .filter(ec_dsl::entry_version.eq(version))
        .load::<models::EntryCategoryRelation>(conn)?
        .into_iter()
        .map(|r| r.category_id)
        .collect();

    let tags = et_dsl::entry_tag_relations
        .filter(et_dsl::entry_id.eq(&id))
        .filter(et_dsl::entry_version.eq(version))
        .load::<models::EntryTagRelation>(conn)?
        .into_iter()
        .map(|r| r.tag_id)
        .collect();

    Ok(Entry {
        id,
        osm_node: osm_node.map(|x| x as u64),
        created: created.into(),
        archived: archived.map(Into::into),
        version: version as u64,
        title,
        description,
        location,
        contact: Some(Contact { email, telephone }),
        homepage,
        categories,
        tags,
        license,
        image_url,
        image_link_url,
    })
}

#[derive(QueryableByName)]
struct TagIdCount {
    #[sql_type = "diesel::sql_types::Text"]
    tag_id: String,

    #[sql_type = "diesel::sql_types::BigInt"]
    count: i64,
}

impl EntryGateway for SqliteConnection {
    fn create_entry(&self, e: Entry) -> Result<()> {
        let cat_rels: Vec<_> = e
            .categories
            .iter()
            .map(|category_id| models::StoreableEntryCategoryRelation {
                entry_id: &e.id,
                entry_version: e.version as i64,
                category_id: &category_id,
            })
            .collect();
        let tag_rels: Vec<_> = e
            .tags
            .iter()
            .map(|tag_id| models::StoreableEntryTagRelation {
                entry_id: &e.id,
                entry_version: e.version as i64,
                tag_id: &tag_id,
            })
            .collect();
        let new_entry = models::Entry::from(e.clone());
        reset_current_entry_before_insert_new_version(self, &new_entry.id)?;
        diesel::insert_into(schema::entries::table)
            .values(&new_entry)
            .execute(self)?;
        diesel::insert_into(schema::entry_category_relations::table)
            //WHERE NOT EXISTS
            .values(&cat_rels)
            .execute(self)?;
        diesel::insert_into(schema::entry_tag_relations::table)
            //WHERE NOT EXISTS
            .values(&tag_rels)
            .execute(self)?;
        Ok(())
    }

    fn get_entry(&self, id: &str) -> Result<Entry> {
        use self::schema::entries::dsl as e_dsl;

        let entry = e_dsl::entries
            .filter(e_dsl::id.eq(id))
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::archived.is_null())
            .first(self)?;

        load_entry(self, entry)
    }

    fn get_entries(&self, ids: &[&str]) -> Result<Vec<Entry>> {
        use self::schema::entries::dsl as e_dsl;

        // TODO: Split loading into chunks of fixed size
        info!("Loading multiple ({}) entries at once", ids.len());
        let entries = e_dsl::entries
            .filter(e_dsl::id.eq_any(ids))
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::archived.is_null())
            .load::<models::Entry>(self)?;

        let mut results = Vec::with_capacity(entries.len());
        for entry in entries {
            results.push(load_entry(self, entry)?);
        }
        Ok(results)
    }

    fn all_entries(&self) -> Result<Vec<Entry>> {
        use self::schema::{
            entries::dsl as e_dsl, entry_category_relations::dsl as ec_dsl,
            entry_tag_relations::dsl as et_dsl,
        };
        // TODO: Don't load all table contents into memory at once!
        // We only need to iterator over the sorted rows of the results.
        // Unfortunately Diesel does not offer a Cursor API.
        let entries: Vec<models::Entry> = e_dsl::entries
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::id)
            .load(self)?;
        let mut cat_rels = ec_dsl::entry_category_relations
            .order_by(ec_dsl::entry_id)
            .load(self)?
            .into_iter()
            .peekable();
        let mut tag_rels = et_dsl::entry_tag_relations
            .order_by(et_dsl::entry_id)
            .load(self)?
            .into_iter()
            .peekable();
        let mut res_entries = Vec::with_capacity(entries.len());
        // All results are sorted by entry id and we only need to iterate
        // once through the results to pick up the categories and tags of
        // each entry.
        for entry in entries.into_iter() {
            let mut categories = Vec::with_capacity(10);
            let mut tags = Vec::with_capacity(100);
            // Skip orphaned/deleted relations for which no current entry exists
            while let Some(true) = cat_rels
                .peek()
                .map(|ec: &models::EntryCategoryRelation| ec.entry_id < entry.id)
            {
                let _ = cat_rels.next();
            }
            while let Some(true) = tag_rels
                .peek()
                .map(|et: &models::EntryTagRelation| et.entry_id < entry.id)
            {
                let _ = tag_rels.next();
            }
            // Collect categories of the current entry
            while let Some(true) = cat_rels
                .peek()
                .map(|ec: &models::EntryCategoryRelation| ec.entry_id == entry.id)
            {
                let cat_rel = cat_rels.next().unwrap();
                if cat_rel.entry_version != entry.version {
                    // Skip category relation with outdated version
                    debug_assert!(cat_rel.entry_version < entry.version);
                } else {
                    categories.push(cat_rel.category_id);
                }
            }
            // Collect tags of entry
            while let Some(true) = tag_rels
                .peek()
                .map(|et: &models::EntryTagRelation| et.entry_id == entry.id)
            {
                let tag_rel = tag_rels.next().unwrap();
                if tag_rel.entry_version != entry.version {
                    // Skip tag relation with outdated version
                    debug_assert!(tag_rel.entry_version < entry.version);
                } else {
                    tags.push(tag_rel.tag_id);
                }
            }
            // Convert into result entry
            res_entries.push((entry, categories, tags).into());
        }
        Ok(res_entries)
    }

    fn recently_changed_entries(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<Entry>> {
        use self::schema::{
            entries::dsl as e_dsl, entry_category_relations::dsl as ec_dsl,
            entry_tag_relations::dsl as et_dsl,
        };
        // TODO: Don't load all table contents into memory at once!
        // We only need to iterator over the sorted rows of the results.
        // Unfortunately Diesel does not offer a Cursor API.
        let changed_expr = diesel::dsl::sql::<diesel::sql_types::BigInt>(
            "COALESCE(entries.archived, entries.created)",
        );
        let mut query = e_dsl::entries
            .filter(e_dsl::current.eq(true))
            .order_by(changed_expr.clone().desc())
            .then_order_by(e_dsl::id) // disambiguation if time stamps are equal
            .into_boxed();
        let mut cats_query = e_dsl::entries
            .left_outer_join(
                ec_dsl::entry_category_relations.on(ec_dsl::entry_id
                    .eq(e_dsl::id)
                    .and(ec_dsl::entry_version.eq(e_dsl::version))),
            )
            .select((e_dsl::id, e_dsl::version, ec_dsl::category_id.nullable()))
            .order_by(changed_expr.clone().desc())
            .then_order_by(e_dsl::id)
            .into_boxed();
        let mut tags_query = e_dsl::entries
            .left_outer_join(
                et_dsl::entry_tag_relations.on(et_dsl::entry_id
                    .eq(e_dsl::id)
                    .and(et_dsl::entry_version.eq(e_dsl::version))),
            )
            .select((e_dsl::id, e_dsl::version, et_dsl::tag_id.nullable()))
            .order_by(changed_expr.clone().desc())
            .then_order_by(e_dsl::id)
            .into_boxed();
        if let Some(since) = params.since {
            // inclusive
            query = query.filter(changed_expr.clone().ge(i64::from(since)));
            cats_query = cats_query.filter(changed_expr.clone().ge(i64::from(since)));
            tags_query = tags_query.filter(changed_expr.clone().ge(i64::from(since)));
        }
        if let Some(until) = params.until {
            // exclusive
            query = query.filter(changed_expr.clone().lt(i64::from(until)));
            cats_query = cats_query.filter(changed_expr.clone().lt(i64::from(until)));
            tags_query = tags_query.filter(changed_expr.clone().lt(i64::from(until)));
        }
        // Pagination must only be applied to the entries query!
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            query = query.offset(offset as i64);
        }
        if let Some(limit) = pagination.limit {
            query = query.limit(limit as i64);
        }
        let entries: Vec<models::Entry> = query.load(self)?;
        let mut cat_rels = cats_query
            .load::<(String, i64, Option<String>)>(self)?
            .into_iter()
            .peekable();
        let mut tag_rels = tags_query
            .load::<(String, i64, Option<String>)>(self)?
            .into_iter()
            .peekable();
        let mut res_entries = Vec::with_capacity(entries.len());
        // All results are sorted according to the same criteria.
        // The categories/tags queries may contains results for
        // additional entries due to missing pagination. Those
        // results will be skipped during the merge phase.
        for entry in entries.into_iter() {
            // Ignore categories of other entries
            while let Some(true) = cat_rels
                .peek()
                .map(|(id, version, _)| &entry.id != id || &entry.version != version)
            {
                cat_rels.next().unwrap();
            }
            // Collect categories of current entry
            let mut categories = Vec::with_capacity(4);
            while let Some(true) = cat_rels
                .peek()
                .map(|(id, version, _)| &entry.id == id && &entry.version == version)
            {
                if let (_, _, Some(cat)) = cat_rels.next().unwrap() {
                    categories.push(cat);
                }
            }
            // Ignore tags of other entries
            while let Some(true) = tag_rels
                .peek()
                .map(|(id, version, _)| &entry.id != id || &entry.version != version)
            {
                tag_rels.next().unwrap();
            }
            // Collect tags of current entry
            let mut tags = Vec::with_capacity(30);
            while let Some(true) = tag_rels
                .peek()
                .map(|(id, version, _)| &entry.id == id && &entry.version == version)
            {
                if let (_, _, Some(tag)) = tag_rels.next().unwrap() {
                    tags.push(tag);
                }
            }
            // Convert into resulting entry
            res_entries.push((entry, categories, tags).into());
        }
        Ok(res_entries)
    }

    fn most_popular_entry_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        // TODO: Diesel 1.4.x does not support the HAVING clause
        // that is required to filter the aggregated column.
        let mut sql = "SELECT tag_id, COUNT(*) as count \
                       FROM entry_tag_relations \
                       WHERE (entry_id, entry_version) IN \
                       (SELECT id, version FROM entries WHERE current=1 AND archived IS NULL) \
                       GROUP BY tag_id"
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
        sql.push_str(" ORDER BY count DESC, tag_id");
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        if let Some(limit) = pagination.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        let rows = diesel::dsl::sql_query(sql).load::<TagIdCount>(self)?;
        Ok(rows
            .into_iter()
            .map(|row| TagFrequency(row.tag_id, row.count as TagCount))
            .collect())
    }

    fn count_entries(&self) -> Result<usize> {
        use self::schema::entries::dsl as e_dsl;
        Ok(e_dsl::entries
            .select(diesel::dsl::count(e_dsl::id))
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::archived.is_null())
            .first::<i64>(self)? as usize)
    }

    fn update_entry(&self, entry: &Entry) -> Result<()> {
        let e = models::Entry::from(entry.clone());

        let cat_rels: Vec<_> = entry
            .categories
            .iter()
            .map(|category_id| models::StoreableEntryCategoryRelation {
                entry_id: &entry.id,
                entry_version: entry.version as i64,
                category_id: &category_id,
            })
            .collect();

        let tag_rels: Vec<_> = entry
            .tags
            .iter()
            .map(|tag_id| models::StoreableEntryTagRelation {
                entry_id: &entry.id,
                entry_version: entry.version as i64,
                tag_id: &tag_id,
            })
            .collect();

        reset_current_entry_before_insert_new_version(self, &e.id)?;
        diesel::insert_into(schema::entries::table)
            .values(&e)
            .execute(self)?;
        diesel::insert_into(schema::entry_category_relations::table)
            //WHERE NOT EXISTS
            .values(&cat_rels)
            .execute(self)?;
        diesel::insert_into(schema::entry_tag_relations::table)
            //WHERE NOT EXISTS
            .values(&tag_rels)
            .execute(self)?;
        Ok(())
    }

    fn archive_entries(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        // Entry
        use self::schema::entries::dsl as e_dsl;
        let count = diesel::update(
            e_dsl::entries
                .filter(e_dsl::id.eq_any(ids))
                .filter(e_dsl::current.eq(true))
                .filter(e_dsl::archived.is_null()),
        )
        .set(e_dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        if count < ids.len() {
            return Err(RepoError::NotFound);
        }
        if count > ids.len() {
            // Should never happen
            return Err(RepoError::TooManyFound);
        }
        Ok(count)
    }

    fn import_multiple_entries(&mut self, new_entries: &[Entry]) -> Result<()> {
        let imports: Vec<_> = new_entries
            .iter()
            .map(|e| {
                let new_entry = models::Entry::from(e.clone());
                let cat_rels: Vec<_> = e
                    .categories
                    .iter()
                    .map(|category_id| models::StoreableEntryCategoryRelation {
                        entry_id: &e.id,
                        entry_version: e.version as i64,
                        category_id: &category_id,
                    })
                    .collect();
                let tag_rels: Vec<_> = e
                    .tags
                    .iter()
                    .map(|tag_id| models::StoreableEntryTagRelation {
                        entry_id: &e.id,
                        entry_version: e.version as i64,
                        tag_id: &tag_id,
                    })
                    .collect();
                (new_entry, cat_rels, tag_rels)
            })
            .collect();
        self.transaction::<_, diesel::result::Error, _>(|| {
            for (new_entry, cat_rels, tag_rels) in imports {
                reset_current_entry_before_insert_new_version(self, &new_entry.id).map_err(
                    |err| {
                        error!(
                            "Import: Failed to reset current entry {}: {}",
                            new_entry.id, err
                        );
                        diesel::result::Error::RollbackTransaction
                    },
                )?;
                diesel::insert_into(schema::entries::table)
                    .values(&new_entry)
                    .execute(self)?;
                diesel::insert_into(schema::entry_category_relations::table)
                    .values(&cat_rels)
                    .execute(self)?;

                for r in &tag_rels {
                    let res = diesel::insert_into(schema::tags::table)
                        .values(&models::Tag {
                            id: r.tag_id.to_owned(),
                        })
                        .execute(self);
                    if let Err(err) = res {
                        match err {
                            DieselError::DatabaseError(db_err, _) => {
                                match db_err {
                                    DatabaseErrorKind::UniqueViolation => {
                                        // that's ok :)
                                    }
                                    _ => {
                                        return Err(err);
                                    }
                                }
                            }
                            _ => {
                                return Err(err);
                            }
                        }
                    }
                }
                diesel::insert_into(schema::entry_tag_relations::table)
                    .values(&tag_rels)
                    .execute(self)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

fn into_new_event_with_tags(
    db: &SqliteConnection,
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
        (c.email, c.telephone)
    } else {
        (None, None)
    };

    let registration = registration.map(Into::into);

    let created_by = if let Some(ref email) = created_by {
        Some(resolve_user_id_by_email(db, email)?)
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
            email,
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

fn resolve_event_id(db: &SqliteConnection, uid: &str) -> Result<i64> {
    use self::schema::events::dsl;
    Ok(dsl::events
        .select(dsl::id)
        .filter(dsl::uid.eq(uid))
        .first(db)?)
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
            use self::schema::event_tags::dsl as et_dsl;
            use self::schema::events::dsl as e_dsl;
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
        use self::schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};

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
            Some(Contact { email, telephone })
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
        use self::schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};
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
        use self::schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl, users::dsl as u_dsl};
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
        use self::schema::events::dsl;
        Ok(dsl::events
            .select(diesel::dsl::count(dsl::id))
            .filter(dsl::archived.is_null())
            .first::<i64>(self)? as usize)
    }

    fn archive_events(&self, uids: &[&str], archived: Timestamp) -> Result<usize> {
        use self::schema::events::dsl;
        let count = diesel::update(
            dsl::events
                .filter(dsl::uid.eq_any(uids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?;
        debug_assert!(count <= uids.len());
        if count < uids.len() {
            return Err(RepoError::NotFound);
        }
        if count > uids.len() {
            return Err(RepoError::TooManyFound);
        }
        Ok(count)
    }

    fn delete_event_with_matching_tags(&self, uid: &str, tags: &[&str]) -> Result<Option<()>> {
        use self::schema::{event_tags::dsl as et_dsl, events::dsl as e_dsl};
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

fn resolve_user_id_by_email(db: &SqliteConnection, email: &str) -> Result<i64> {
    use self::schema::users::dsl;
    Ok(dsl::users
        .select(dsl::id)
        .filter(dsl::email.eq(email))
        .first(db)?)
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
        use self::schema::users::dsl;
        let new_user = models::NewUser::from(u);
        diesel::update(dsl::users.filter(dsl::email.eq(new_user.email)))
            .set(&new_user)
            .execute(self)?;
        Ok(())
    }

    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        use self::schema::users::dsl;
        diesel::delete(dsl::users.filter(dsl::email.eq(email))).execute(self)?;
        Ok(())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)?
            .into())
    }

    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .filter(dsl::email.eq(email))
            .first::<models::UserEntity>(self)
            .optional()?
            .map(Into::into))
    }

    fn all_users(&self) -> Result<Vec<User>> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .load::<models::UserEntity>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn count_users(&self) -> Result<usize> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .select(diesel::dsl::count(dsl::id))
            .first::<i64>(self)? as usize)
    }
}

impl RatingRepository for SqliteConnection {
    fn create_rating(&self, rating: Rating) -> Result<()> {
        diesel::insert_into(schema::ratings::table)
            .values(&models::Rating::from(rating))
            .execute(self)?;
        Ok(())
    }

    fn load_rating(&self, id: &str) -> Result<Rating> {
        use self::schema::ratings::dsl;
        Ok(dsl::ratings
            .filter(dsl::id.eq(id))
            .filter(dsl::archived.is_null())
            .first::<models::Rating>(self)?
            .into())
    }

    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>> {
        use self::schema::ratings::dsl;
        // TODO: Split loading into chunks of fixed size
        info!("Loading multiple ({}) ratings at once", ids.len());
        Ok(dsl::ratings
            .filter(dsl::id.eq_any(ids))
            .filter(dsl::archived.is_null())
            .load::<models::Rating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_ratings_of_entry(&self, entry_id: &str) -> Result<Vec<Rating>> {
        use self::schema::ratings::dsl;
        Ok(dsl::ratings
            .filter(dsl::entry_id.eq(entry_id))
            .filter(dsl::archived.is_null())
            .load::<models::Rating>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_entry_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>> {
        use self::schema::ratings::dsl;
        Ok(dsl::ratings
            .select(dsl::entry_id)
            .distinct()
            .filter(dsl::id.eq_any(ids))
            .load::<String>(self)?)
    }

    fn archive_ratings(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        use self::schema::ratings::dsl as r_dsl;
        let count = diesel::update(
            r_dsl::ratings
                .filter(r_dsl::id.eq_any(ids))
                .filter(r_dsl::archived.is_null()),
        )
        .set(r_dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        if count < ids.len() {
            return Err(RepoError::NotFound);
        }
        if count > ids.len() {
            // Should never happen
            return Err(RepoError::TooManyFound);
        }
        Ok(count)
    }

    fn archive_ratings_of_entries(&self, entry_ids: &[&str], archived: Timestamp) -> Result<usize> {
        use self::schema::ratings::dsl;
        Ok(diesel::update(
            dsl::ratings
                .filter(dsl::entry_id.eq_any(entry_ids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?)
    }
}

impl CommentRepository for SqliteConnection {
    fn create_comment(&self, c: Comment) -> Result<()> {
        diesel::insert_into(schema::comments::table)
            .values(&models::Comment::from(c))
            .execute(self)?;
        Ok(())
    }

    fn load_comment(&self, id: &str) -> Result<Comment> {
        use self::schema::comments::dsl;
        Ok(dsl::comments
            .filter(dsl::id.eq(id))
            .filter(dsl::archived.is_null())
            .first::<models::Comment>(self)?
            .into())
    }

    fn load_comments(&self, ids: &[&str]) -> Result<Vec<Comment>> {
        use self::schema::comments::dsl;
        // TODO: Split loading into chunks of fixed size
        info!("Loading multiple ({}) comments at once", ids.len());
        Ok(dsl::comments
            .filter(dsl::id.eq_any(ids))
            .filter(dsl::archived.is_null())
            .load::<models::Comment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>> {
        use self::schema::comments::dsl;
        Ok(dsl::comments
            .filter(dsl::rating_id.eq(rating_id))
            .filter(dsl::archived.is_null())
            .load::<models::Comment>(self)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn archive_comments(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        use self::schema::comments::dsl;
        let count = diesel::update(
            dsl::comments
                .filter(dsl::id.eq_any(ids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?;
        debug_assert!(count <= ids.len());
        if count < ids.len() {
            return Err(RepoError::NotFound);
        }
        if count > ids.len() {
            return Err(RepoError::TooManyFound);
        }
        Ok(count)
    }

    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        archived: Timestamp,
    ) -> Result<usize> {
        use self::schema::comments::dsl;
        Ok(diesel::update(
            dsl::comments
                .filter(dsl::rating_id.eq_any(rating_ids))
                .filter(dsl::archived.is_null()),
        )
        .set(dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?)
    }

    fn archive_comments_of_entries(
        &self,
        entry_ids: &[&str],
        archived: Timestamp,
    ) -> Result<usize> {
        use self::schema::comments::dsl as c_dsl;
        use self::schema::ratings::dsl as r_dsl;
        Ok(diesel::update(
            c_dsl::comments
                .filter(
                    c_dsl::rating_id.eq_any(
                        r_dsl::ratings
                            .select(r_dsl::id)
                            .filter(r_dsl::entry_id.eq_any(entry_ids)),
                    ),
                )
                .filter(c_dsl::archived.is_null()),
        )
        .set(c_dsl::archived.eq(Some(i64::from(archived))))
        .execute(self)?)
    }
}

impl Db for SqliteConnection {
    fn create_tag_if_it_does_not_exist(&self, t: &Tag) -> Result<()> {
        let res = diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self);
        if let Err(err) = res {
            match err {
                DieselError::DatabaseError(db_err, _) => {
                    match db_err {
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
    fn create_category_if_it_does_not_exist(&mut self, c: &Category) -> Result<()> {
        let res = diesel::insert_into(schema::categories::table)
            .values(&models::Category::from(c.clone()))
            .execute(self);
        if let Err(err) = res {
            match err {
                DieselError::DatabaseError(db_err, _) => {
                    match db_err {
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
        use self::schema::bbox_subscriptions::dsl as s_dsl;
        use self::schema::users::dsl as u_dsl;
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
        use self::schema::bbox_subscriptions::dsl as s_dsl;
        use self::schema::users::dsl as u_dsl;
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
        use self::schema::bbox_subscriptions::dsl as s_dsl;
        use self::schema::users::dsl as u_dsl;
        let users_id = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(email));
        diesel::delete(s_dsl::bbox_subscriptions.filter(s_dsl::user_id.eq_any(users_id)))
            .execute(self)?;
        Ok(())
    }
    fn all_categories(&self) -> Result<Vec<Category>> {
        use self::schema::categories::dsl::*;
        Ok(categories
            .load::<models::Category>(self)?
            .into_iter()
            .map(Category::from)
            .collect())
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use self::schema::tags::dsl::*;
        Ok(tags
            .load::<models::Tag>(self)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn count_tags(&self) -> Result<usize> {
        use self::schema::tags::dsl::*;
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
        use self::schema::{org_tag_relations::dsl as o_t_dsl, organizations::dsl as o_dsl};

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
        use self::schema::org_tag_relations::dsl;
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
        use self::schema::user_tokens::dsl;
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
            diesel::update(schema::user_tokens::table)
                .filter(dsl::user_id.eq(model.user_id))
                .set(&model)
                .execute(self)?;
        }
        Ok(token.email_nonce)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        use self::schema::user_tokens::dsl as t_dsl;
        use self::schema::users::dsl as u_dsl;
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
        use self::schema::user_tokens::dsl;
        Ok(diesel::delete(
            dsl::user_tokens.filter(dsl::expires_at.lt::<i64>(expired_before.into())),
        )
        .execute(self)?)
    }

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken> {
        use self::schema::user_tokens::dsl as t_dsl;
        use self::schema::users::dsl as u_dsl;
        Ok(t_dsl::user_tokens
            .inner_join(u_dsl::users)
            .select((u_dsl::id, t_dsl::nonce, t_dsl::expires_at, u_dsl::email))
            .filter(u_dsl::email.eq(email))
            .first::<models::UserTokenEntity>(self)?
            .into())
    }
}
