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

fn reset_current_entry_mut(
    conn: &&mut SqliteConnection,
    id: &str,
) -> result::Result<usize, diesel::result::Error> {
    use self::schema::entries::dsl;
    diesel::update(
        dsl::entries
            .filter(dsl::id.eq(id))
            .filter(dsl::current.eq(true)),
    )
    .set(dsl::current.eq(false))
    .execute(*conn)
}

fn reset_current_entry(
    conn: &&SqliteConnection,
    id: &str,
) -> result::Result<usize, diesel::result::Error> {
    use self::schema::entries::dsl;
    diesel::update(
        dsl::entries
            .filter(dsl::id.eq(id))
            .filter(dsl::current.eq(true)),
    )
    .set(dsl::current.eq(false))
    .execute(*conn)
}

impl EntryGateway for SqliteConnection {
    fn create_entry(&self, e: Entry) -> Result<()> {
        let cat_rels: Vec<_> = e
            .categories
            .iter()
            .cloned()
            .map(|category_id| models::EntryCategoryRelation {
                entry_id: e.id.clone(),
                entry_version: e.version as i64,
                category_id,
            })
            .collect();
        let tag_rels: Vec<_> = e
            .tags
            .iter()
            .cloned()
            .map(|tag_id| models::EntryTagRelation {
                entry_id: e.id.clone(),
                entry_version: e.version as i64,
                tag_id,
            })
            .collect();
        let new_entry = models::Entry::from(e);
        reset_current_entry(&self, &new_entry.id)?;
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
        use self::schema::entry_category_relations::dsl as e_c_dsl;
        use self::schema::entry_tag_relations::dsl as e_t_dsl;

        let models::Entry {
            id,
            osm_node,
            created,
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
        } = e_dsl::entries
            .filter(e_dsl::id.eq(id))
            .filter(e_dsl::current.eq(true))
            .first(self)?;

        let location = Location {
            lat: lat as f64,
            lng: lng as f64,
            address: Some(Address {
                street,
                zip,
                city,
                country,
            }),
        };

        let categories = e_c_dsl::entry_category_relations
            .filter(
                e_c_dsl::entry_id
                    .eq(&id)
                    .and(e_c_dsl::entry_version.eq(version)),
            )
            .load::<models::EntryCategoryRelation>(self)?
            .into_iter()
            .map(|r| r.category_id)
            .collect();

        let tags = e_t_dsl::entry_tag_relations
            .filter(
                e_t_dsl::entry_id
                    .eq(&id)
                    .and(e_t_dsl::entry_version.eq(version)),
            )
            .load::<models::EntryTagRelation>(self)?
            .into_iter()
            .map(|r| r.tag_id)
            .collect();

        Ok(Entry {
            id,
            osm_node: osm_node.map(|x| x as u64),
            created: created as u64,
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

    fn count_entries(&self) -> Result<usize> {
        use self::schema::entries::dsl as e_dsl;
        let count: i64 = e_dsl::entries
            .select(diesel::dsl::count(e_dsl::id))
            .filter(e_dsl::current.eq(true))
            .first(self)?;
        Ok(count as usize)
    }

    fn update_entry(&self, entry: &Entry) -> Result<()> {
        let e = models::Entry::from(entry.clone());

        let cat_rels: Vec<_> = entry
            .categories
            .iter()
            .cloned()
            .map(|category_id| models::EntryCategoryRelation {
                entry_id: entry.id.clone(),
                entry_version: entry.version as i64,
                category_id,
            })
            .collect();

        let tag_rels: Vec<_> = entry
            .tags
            .iter()
            .cloned()
            .map(|tag_id| models::EntryTagRelation {
                entry_id: entry.id.clone(),
                entry_version: entry.version as i64,
                tag_id,
            })
            .collect();

        reset_current_entry(&self, &e.id)?;
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

    fn import_multiple_entries(&mut self, new_entries: &[Entry]) -> Result<()> {
        let imports: Vec<_> = new_entries
            .into_iter()
            .map(|e| {
                let new_entry = models::Entry::from(e.clone());
                let cat_rels: Vec<_> = e
                    .categories
                    .iter()
                    .cloned()
                    .map(|category_id| models::EntryCategoryRelation {
                        entry_id: e.id.clone(),
                        entry_version: e.version as i64,
                        category_id,
                    })
                    .collect();
                let tag_rels: Vec<_> = e
                    .tags
                    .iter()
                    .map(|tag_id| models::EntryTagRelation {
                        entry_id: e.id.clone(),
                        entry_version: e.version as i64,
                        tag_id: tag_id.clone(),
                    })
                    .collect();
                (new_entry, cat_rels, tag_rels)
            })
            .collect();
        self.transaction::<_, diesel::result::Error, _>(|| {
            for (new_entry, cat_rels, tag_rels) in imports {
                reset_current_entry_mut(&self, &new_entry.id)?;
                diesel::insert_into(schema::entries::table)
                    .values(&new_entry)
                    .execute(self)?;
                diesel::insert_into(schema::entry_category_relations::table)
                    .values(&cat_rels)
                    .execute(self)?;

                for r in &tag_rels {
                    let res = diesel::insert_into(schema::tags::table)
                        .values(&models::Tag {
                            id: r.tag_id.clone(),
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
    fn create_event(&mut self, e: Event) -> Result<()> {
        let tag_rels: Vec<_> = e
            .tags
            .iter()
            .cloned()
            .map(|tag_id| models::EventTagRelation {
                event_id: e.id.clone(),
                tag_id,
            })
            .collect();
        let new_event = models::Event::from(e);
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
        } = e_dsl::events.filter(e_dsl::id.eq(e_id)).first(self)?;

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

        let location = if lat.is_some() || lng.is_some() || address.is_some() {
            Some(Location {
                // TODO: How to handle missing lat/lng?
                lat: lat.map(|x| x as f64).unwrap_or(0.0),
                lng: lng.map(|x| x as f64).unwrap_or(0.0),
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

        let registration = registration.map(|x| x.into());

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
        })
    }

    fn all_events(&self) -> Result<Vec<Event>> {
        use self::schema::{event_tag_relations::dsl as e_t_dsl, events::dsl as e_dsl};
        let events: Vec<models::Event> = e_dsl::events.load(self)?;
        let tag_rels = e_t_dsl::event_tag_relations.load(self)?;
        Ok(events.into_iter().map(|e| (e, &tag_rels).into()).collect())
    }

    fn update_event(&mut self, event: &Event) -> Result<()> {
        let e = models::Event::from(event.clone());
        self.transaction::<_, diesel::result::Error, _>(|| {
            use self::schema::event_tag_relations::dsl as e_t_dsl;
            use self::schema::events::dsl as e_dsl;

            let old_tags = self.get_event(&e.id).unwrap().tags;
            let new_tags = &event.tags;
            let diff = super::util::tags_diff(&old_tags, new_tags);

            let tag_rels: Vec<_> = diff
                .added
                .into_iter()
                .map(|tag_id| models::EventTagRelation {
                    event_id: event.id.clone(),
                    tag_id,
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

    fn delete_event(&mut self, id: &str) -> Result<()> {
        use self::schema::events::dsl;
        diesel::delete(dsl::events.filter(dsl::id.eq(id))).execute(self)?;
        Ok(())
    }
}

impl UserGateway for SqliteConnection {
    fn create_user(&mut self, u: User) -> Result<()> {
        diesel::insert_into(schema::users::table)
            .values(&models::User::from(u))
            .execute(self)?;
        Ok(())
    }
    fn update_user(&mut self, u: &User) -> Result<()> {
        use self::schema::users::dsl;
        let user = models::User::from(u.clone());
        diesel::update(dsl::users.filter(dsl::id.eq(&u.id)))
            .set(&user)
            .execute(self)?;
        Ok(())
    }
    fn get_user(&self, username: &str) -> Result<User> {
        use self::schema::users::dsl::users;
        let u: models::User = users.find(username).first(self)?;
        Ok(User::from(u))
    }
    fn all_users(&self) -> Result<Vec<User>> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .load::<models::User>(self)?
            .into_iter()
            .map(User::from)
            .collect())
    }
    fn delete_user(&mut self, user_name: &str) -> Result<()> {
        use self::schema::users::dsl::*;
        diesel::delete(users.find(user_name)).execute(self)?;
        Ok(())
    }
}

impl CommentGateway for SqliteConnection {
    fn create_comment(&self, c: Comment) -> Result<()> {
        diesel::insert_into(schema::comments::table)
            .values(&models::Comment::from(c))
            .execute(self)?;
        Ok(())
    }
    fn all_comments(&self) -> Result<Vec<Comment>> {
        use self::schema::comments::dsl::*;
        Ok(comments
            .load::<models::Comment>(self)?
            .into_iter()
            .map(Comment::from)
            .collect())
    }
}

impl RatingRepository for SqliteConnection {
    fn get_rating(&self, id: &str) -> Result<Rating> {
        use self::schema::ratings::dsl;
        dsl::ratings
            .filter(dsl::id.eq(id))
            .first::<models::Rating>(self)
            .map(Rating::from)
            .map_err(Into::into)
    }

    fn get_ratings(&self, ids: &[String]) -> Result<Vec<Rating>> {
        let mut ratings = Vec::with_capacity(ids.len());
        // TODO: Load multiple ratings at once in batches of limited size
        for id in ids {
            match self.get_rating(id) {
                Ok(rating) => {
                    // Success
                    ratings.push(rating);
                }
                Err(RepoError::NotFound) => {
                    // Some of the requested ratings might not exist
                    info!("One of multiple ratings not found: {}", id);
                }
                Err(err) => {
                    // Abort on any unexpected error
                    return Err(err);
                }
            }
        }
        Ok(ratings)
    }

    fn all_ratings_for_entry_by_id(&self, entry_id: &str) -> Result<Vec<Rating>> {
        use self::schema::ratings::dsl;
        Ok(dsl::ratings
            .filter(dsl::entry_id.eq(entry_id))
            .load::<models::Rating>(self)?
            .into_iter()
            .map(Rating::from)
            .collect())
    }

    fn add_rating_for_entry(&self, rating: Rating) -> Result<()> {
        diesel::insert_into(schema::ratings::table)
            .values(&models::Rating::from(rating))
            .execute(self)?;
        Ok(())
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
        let count: i64 = tags.select(diesel::dsl::count(id)).first(self)?;
        Ok(count as usize)
    }
}

impl OrganizationGateway for SqliteConnection {
    fn create_org(&mut self, o: Organization) -> Result<()> {
        let tag_rels: Vec<_> = o
            .owned_tags
            .iter()
            .cloned()
            .map(|tag_id| models::OrgTagRelation {
                org_id: o.id.clone(),
                tag_id,
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
