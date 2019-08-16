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
    use self::schema::entry_category_relations::dsl as e_c_dsl;
    use self::schema::entry_tag_relations::dsl as e_t_dsl;

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

    let categories = e_c_dsl::entry_category_relations
        .filter(e_c_dsl::entry_id.eq(&id))
        .filter(e_c_dsl::entry_version.eq(version))
        .load::<models::EntryCategoryRelation>(conn)?
        .into_iter()
        .map(|r| r.category_id)
        .collect();

    let tags = e_t_dsl::entry_tag_relations
        .filter(e_t_dsl::entry_id.eq(&id))
        .filter(e_t_dsl::entry_version.eq(version))
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
            entries::dsl as e_dsl, entry_category_relations::dsl as e_c_dsl,
            entry_tag_relations::dsl as e_t_dsl,
        };
        // TODO: Don't load all table contents into memory at once!
        // We only need to iterator over the sorted rows of the results.
        // Unfortunately Diesel does not offer a Cursor API.
        let entries: Vec<models::Entry> = e_dsl::entries
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::archived.is_null())
            .order_by(e_dsl::id)
            .load(self)?;
        let mut cat_rels = e_c_dsl::entry_category_relations
            .order_by(e_c_dsl::entry_id)
            .load(self)?
            .into_iter()
            .peekable();
        let mut tag_rels = e_t_dsl::entry_tag_relations
            .order_by(e_t_dsl::entry_id)
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
            entries::dsl as e_dsl, entry_category_relations::dsl as e_c_dsl,
            entry_tag_relations::dsl as e_t_dsl,
        };
        // TODO: Don't load all table contents into memory at once!
        // We only need to iterator over the sorted rows of the results.
        // Unfortunately Diesel does not offer a Cursor API.
        let changed_expr =
            diesel::dsl::sql::<diesel::sql_types::BigInt>("COALESCE(archived, created)");
        let mut query = e_dsl::entries
            .filter(e_dsl::current.eq(true))
            .order_by(changed_expr.clone().desc())
            .then_order_by(e_dsl::id) // disambiguation if time stamps are equal
            .into_boxed();
        if let Some(since) = params.since {
            query = query.filter(changed_expr.clone().ge(i64::from(since))) // inclusive
        }
        if let Some(until) = params.until {
            query = query.filter(changed_expr.clone().lt(i64::from(until))); // exclusive
        }
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            query = query.offset(offset as i64);
        }
        if let Some(limit) = pagination.limit {
            query = query.limit(limit as i64);
        }
        let entries: Vec<models::Entry> = query.load(self)?;
        let mut cat_rels = e_c_dsl::entry_category_relations
            .order_by(e_c_dsl::entry_id)
            .load(self)?
            .into_iter()
            .peekable();
        let mut tag_rels = e_t_dsl::entry_tag_relations
            .order_by(e_t_dsl::entry_id)
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

impl EventGateway for SqliteConnection {
    fn create_event(&self, e: Event) -> Result<()> {
        let tag_rels: Vec<_> = e
            .tags
            .iter()
            .map(|tag_id| models::StoreableEventTagRelation {
                event_id: &e.id,
                tag_id: &tag_id,
            })
            .collect();
        let new_event = models::Event::from(e.clone());
        self.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(schema::events::table)
                .values(&new_event)
                .execute(self)?;
            diesel::insert_into(schema::event_tag_relations::table)
                //WHERE NOT EXISTS
                .values(&tag_rels)
                .execute(self)?;
            Ok(())
        })?;
        Ok(())
    }

    fn get_event(&self, e_id: &str) -> Result<Event> {
        use self::schema::{event_tag_relations::dsl as e_t_dsl, events::dsl as e_dsl};

        let models::Event {
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
            created_by,
            registration,
            organizer,
            archived,
        } = e_dsl::events
            .filter(e_dsl::id.eq(e_id))
            .filter(e_dsl::archived.is_null())
            .first(self)?;

        let tags = e_t_dsl::event_tag_relations
            .filter(e_t_dsl::event_id.eq(&id))
            .load::<models::EventTagRelation>(self)?
            .into_iter()
            .map(|r| r.tag_id)
            .collect();

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
            id,
            title,
            start: NaiveDateTime::from_timestamp(start, 0),
            end: end.map(|x| NaiveDateTime::from_timestamp(x, 0)),
            description,
            location,
            contact,
            homepage,
            tags,
            created_by,
            registration,
            organizer,
            archived: archived.map(Into::into),
        })
    }

    fn all_events(&self) -> Result<Vec<Event>> {
        use self::schema::{event_tag_relations::dsl as e_t_dsl, events::dsl as e_dsl};
        let events: Vec<models::Event> =
            e_dsl::events.filter(e_dsl::archived.is_null()).load(self)?;
        let tag_rels = e_t_dsl::event_tag_relations.load(self)?;
        Ok(events.into_iter().map(|e| (e, &tag_rels).into()).collect())
    }

    fn get_events(
        &self,
        start_min: Option<Timestamp>,
        start_max: Option<Timestamp>,
    ) -> Result<Vec<Event>> {
        use self::schema::{event_tag_relations::dsl as e_t_dsl, events::dsl as e_dsl};
        let mut query = e_dsl::events.filter(e_dsl::archived.is_null()).into_boxed();
        if let Some(start_min) = start_min {
            query = query.filter(e_dsl::start.ge(i64::from(start_min)));
        }
        if let Some(start_max) = start_max {
            query = query.filter(e_dsl::start.le(i64::from(start_max)));
        }
        let events: Vec<models::Event> = query.load(self)?;
        let tag_rels = e_t_dsl::event_tag_relations.load(self)?;
        Ok(events.into_iter().map(|e| (e, &tag_rels).into()).collect())
    }

    fn count_events(&self) -> Result<usize> {
        use self::schema::events::dsl;
        Ok(dsl::events
            .select(diesel::dsl::count(dsl::id))
            .filter(dsl::archived.is_null())
            .first::<i64>(self)? as usize)
    }

    fn update_event(&self, event: &Event) -> Result<()> {
        let e = models::Event::from(event.clone());
        self.transaction::<_, diesel::result::Error, _>(|| {
            use self::schema::event_tag_relations::dsl as e_t_dsl;
            use self::schema::events::dsl as e_dsl;

            let old_tags = match self.get_event(&e.id) {
                Ok(e) => e,
                Err(err) => {
                    warn!(
                        "Cannot update non-existent or archived event '{}': {}",
                        e.id, err
                    );
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            }
            .tags;
            let new_tags = &event.tags;
            let diff = super::util::tags_diff(&old_tags, new_tags);

            let tag_rels: Vec<_> = diff
                .added
                .iter()
                .map(|tag_id| models::StoreableEventTagRelation {
                    event_id: &event.id,
                    tag_id: &tag_id,
                })
                .collect();

            diesel::delete(
                e_t_dsl::event_tag_relations
                    .filter(e_t_dsl::event_id.eq(&e.id))
                    .filter(e_t_dsl::tag_id.eq_any(diff.deleted)),
            )
            .execute(self)?;

            diesel::insert_or_ignore_into(schema::event_tag_relations::table)
                .values(&tag_rels)
                .execute(self)?;

            diesel::update(e_dsl::events.filter(e_dsl::id.eq(&e.id)))
                .set(&e)
                .execute(self)?;

            Ok(())
        })?;
        Ok(())
    }

    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        use self::schema::events::dsl;
        let count = diesel::update(
            dsl::events
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

    fn delete_event(&self, id: &str) -> Result<()> {
        use self::schema::events::dsl;
        diesel::delete(dsl::events.filter(dsl::id.eq(id))).execute(self)?;
        Ok(())
    }
}

impl UserGateway for SqliteConnection {
    fn create_user(&self, u: User) -> Result<()> {
        diesel::insert_into(schema::users::table)
            .values(&models::User::from(u))
            .execute(self)?;
        Ok(())
    }
    fn update_user(&self, u: &User) -> Result<()> {
        use self::schema::users::dsl;
        let user = models::User::from(u.clone());
        diesel::update(dsl::users.filter(dsl::id.eq(&u.id)))
            .set(&user)
            .execute(self)?;
        Ok(())
    }
    fn get_user(&self, username: &str) -> Result<User> {
        use self::schema::users::dsl::users;
        Ok(users.find(username).first::<models::User>(self)?.into())
    }
    fn get_users_by_email(&self, email: &str) -> Result<Vec<User>> {
        use self::schema::users::dsl;
        let users = dsl::users
            .filter(dsl::email.eq(email))
            .load::<models::User>(self)?;
        if users.is_empty() {
            return Err(RepoError::NotFound);
        }
        Ok(users.into_iter().map(User::from).collect())
    }

    fn all_users(&self) -> Result<Vec<User>> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .load::<models::User>(self)?
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

    fn delete_user(&self, user_name: &str) -> Result<()> {
        use self::schema::users::dsl::*;
        diesel::delete(users.find(user_name)).execute(self)?;
        Ok(())
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
    fn create_bbox_subscription(&mut self, sub: &BboxSubscription) -> Result<()> {
        diesel::insert_into(schema::bbox_subscriptions::table)
            .values(&models::BboxSubscription::from(sub.clone()))
            .execute(self)?;
        Ok(())
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use self::schema::bbox_subscriptions::dsl;
        Ok(dsl::bbox_subscriptions
            .load::<models::BboxSubscription>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn delete_bbox_subscription(&mut self, id: &str) -> Result<()> {
        use self::schema::bbox_subscriptions::dsl;
        diesel::delete(dsl::bbox_subscriptions.find(id)).execute(self)?;
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
    fn create_org(&mut self, o: Organization) -> Result<()> {
        let tag_rels: Vec<_> = o
            .owned_tags
            .iter()
            .map(|tag_id| models::StoreableOrgTagRelation {
                org_id: &o.id,
                tag_id: &tag_id,
            })
            .collect();
        let new_org = models::Organization::from(o.clone());
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

impl EmailTokenCredentialsRepository for SqliteConnection {
    fn replace_email_token_credentials(
        &self,
        email_token_credentials: EmailTokenCredentials,
    ) -> Result<EmailTokenCredentials> {
        use self::schema::email_token_credentials::dsl;
        let model = models::NewEmailTokenCredentials::from(&email_token_credentials);
        // Insert...
        if let 0 = diesel::insert_into(schema::email_token_credentials::table)
            .values(&model)
            .execute(self)?
        {
            // ...or update
            diesel::update(schema::email_token_credentials::table)
                .filter(dsl::username.eq(&model.username))
                .set(&model)
                .execute(self)?;
        }
        Ok(email_token_credentials)
    }

    fn consume_email_token_credentials(
        &self,
        email_or_username: &str,
        token: &EmailToken,
    ) -> Result<EmailTokenCredentials> {
        use self::schema::email_token_credentials::dsl;
        let model = dsl::email_token_credentials
            .filter(dsl::nonce.eq(token.nonce.to_string()))
            .filter(dsl::email.eq(token.email.to_string()))
            .filter(
                dsl::username
                    .eq(email_or_username)
                    .or(dsl::email.eq(email_or_username)),
            )
            .first::<models::EmailTokenCredentials>(self)?;
        diesel::delete(dsl::email_token_credentials.filter(dsl::id.eq(model.id))).execute(self)?;
        Ok(model.into())
    }

    fn discard_expired_email_token_credentials(&self, expired_before: Timestamp) -> Result<usize> {
        use self::schema::email_token_credentials::dsl;
        Ok(diesel::delete(
            dsl::email_token_credentials.filter(dsl::expires_at.lt::<i64>(expired_before.into())),
        )
        .execute(self)?)
    }

    #[cfg(test)]
    fn get_email_token_credentials_by_email_or_username(
        &self,
        email_or_username: &str,
    ) -> Result<EmailTokenCredentials> {
        use self::schema::email_token_credentials::dsl;
        let model = dsl::email_token_credentials
            .filter(
                dsl::username
                    .eq(email_or_username)
                    .or(dsl::email.eq(email_or_username)),
            )
            .first::<models::EmailTokenCredentials>(self)?;
        Ok(model.into())
    }
}
