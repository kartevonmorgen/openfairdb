use entities::*;
use business::error::RepoError;
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::result;
use business::db::Db;
use super::models;
use super::schema;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

type Result<T> = result::Result<T, RepoError>;

fn unset_current_on_all_entries(
    con: &&mut SqliteConnection,
    id: &str,
) -> result::Result<usize, diesel::result::Error> {
    use self::schema::entries::dsl;
    diesel::update(
        dsl::entries
            .filter(dsl::id.eq(id))
            .filter(dsl::current.eq(true)),
    ).set(dsl::current.eq(false))
        .execute(*con)
}

impl Db for SqliteConnection {
    fn create_entry(&mut self, e: &Entry) -> Result<()> {
        let new_entry = models::Entry::from(e.clone());
        let cat_rels: Vec<_> = e.categories
            .iter()
            .cloned()
            .map(|category_id| models::EntryCategoryRelation {
                entry_id: e.id.clone(),
                entry_version: e.version as i64,
                category_id,
            })
            .collect();
        let tag_rels: Vec<_> = e.tags
            .iter()
            .cloned()
            .map(|tag_id| models::EntryTagRelation {
                entry_id: e.id.clone(),
                entry_version: e.version as i64,
                tag_id,
            })
            .collect();
        self.transaction::<_, diesel::result::Error, _>(|| {
            unset_current_on_all_entries(&self, &e.id)?;
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
        })?;
        Ok(())
    }
    fn create_tag_if_it_does_not_exist(&mut self, t: &Tag) -> Result<()> {
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
    fn create_user(&mut self, u: &User) -> Result<()> {
        diesel::insert_into(schema::users::table)
            .values(&models::User::from(u.clone()))
            .execute(self)?;
        Ok(())
    }
    fn create_comment(&mut self, c: &Comment) -> Result<()> {
        diesel::insert_into(schema::comments::table)
            .values(&models::Comment::from(c.clone()))
            .execute(self)?;
        Ok(())
    }
    fn create_rating(&mut self, r: &Rating) -> Result<()> {
        diesel::insert_into(schema::ratings::table)
            .values(&models::Rating::from(r.clone()))
            .execute(self)?;
        Ok(())
    }
    fn create_bbox_subscription(&mut self, sub: &BboxSubscription) -> Result<()> {
        diesel::insert_into(schema::bbox_subscriptions::table)
            .values(&models::BboxSubscription::from(sub.clone()))
            .execute(self)?;
        Ok(())
    }
    fn all_users(&self) -> Result<Vec<User>> {
        use self::schema::users::dsl;
        Ok(dsl::users
            .load::<models::User>(self)?
            .into_iter()
            .map(User::from)
            .collect())
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use self::schema::bbox_subscriptions::dsl;
        Ok(dsl::bbox_subscriptions
            .load::<models::BboxSubscription>(self)?
            .into_iter()
            .map(BboxSubscription::from)
            .collect())
    }
    fn confirm_email_address(&mut self, username: &str) -> Result<User> {
        use self::schema::users::dsl;
        diesel::update(dsl::users.find(username))
            .set(dsl::email_confirmed.eq(true))
            .execute(self)?;
        Ok(self.get_user(username)?)
    }
    fn delete_bbox_subscription(&mut self, id: &str) -> Result<()> {
        use self::schema::bbox_subscriptions::dsl;
        diesel::delete(dsl::bbox_subscriptions.find(id)).execute(self)?;
        Ok(())
    }
    fn delete_user(&mut self, user: &str) -> Result<()> {
        use self::schema::users::dsl::*;
        diesel::delete(users.find(user)).execute(self)?;
        Ok(())
    }

    fn get_entry(&self, e_id: &str) -> Result<Entry> {
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
            ..
        } = e_dsl::entries
            .filter(e_dsl::id.eq(e_id))
            .filter(e_dsl::current.eq(true))
            .first(self)?;

        let categories = e_c_dsl::entry_category_relations
            .filter(e_c_dsl::entry_id.eq(&id))
            .load::<models::EntryCategoryRelation>(self)?
            .into_iter()
            .map(|r| r.category_id)
            .collect();

        let tags = e_t_dsl::entry_tag_relations
            .filter(e_t_dsl::entry_id.eq(&id))
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
            lat: lat as f64,
            lng: lng as f64,
            street,
            zip,
            city,
            country,
            email,
            telephone,
            homepage,
            categories,
            tags,
            license,
        })
    }

    fn get_entries_by_bbox(&self, bbox: &Bbox) -> Result<Vec<Entry>> {
        use self::schema::entries::dsl as e_dsl;
        use self::schema::entry_category_relations::dsl as e_c_dsl;
        use self::schema::entry_tag_relations::dsl as e_t_dsl;

        let entries: Vec<models::Entry> = e_dsl::entries
            .filter(e_dsl::current.eq(true))
            .filter(e_dsl::lat.between(bbox.south_west.lat, bbox.north_east.lat))
            .filter(e_dsl::lng.between(bbox.south_west.lng, bbox.north_east.lng))
            .load(self)?;

        let cat_rels =
            e_c_dsl::entry_category_relations.load::<models::EntryCategoryRelation>(self)?;

        let tag_rels = e_t_dsl::entry_tag_relations.load::<models::EntryTagRelation>(self)?;

        Ok(entries
            .into_iter()
            .map(|e| {
                let cats = cat_rels
                    .iter()
                    .filter(|r| r.entry_id == e.id)
                    .filter(|r| r.entry_version == e.version)
                    .map(|r| &r.category_id)
                    .cloned()
                    .collect();
                let tags = tag_rels
                    .iter()
                    .filter(|r| r.entry_id == e.id)
                    .filter(|r| r.entry_version == e.version)
                    .map(|r| &r.tag_id)
                    .cloned()
                    .collect();
                Entry {
                    id: e.id,
                    osm_node: e.osm_node.map(|x| x as u64),
                    created: e.created as u64,
                    version: e.version as u64,
                    title: e.title,
                    description: e.description,
                    lat: e.lat as f64,
                    lng: e.lng as f64,
                    street: e.street,
                    zip: e.zip,
                    city: e.city,
                    country: e.country,
                    email: e.email,
                    telephone: e.telephone,
                    homepage: e.homepage,
                    categories: cats,
                    tags: tags,
                    license: e.license,
                }
            })
            .collect())
    }

    fn get_user(&self, user_id: &str) -> Result<User> {
        use self::schema::users::dsl::*;
        let u: models::User = users.find(user_id).first(self)?;
        Ok(User::from(u))
    }

    fn all_entries(&self) -> Result<Vec<Entry>> {
        use self::schema::entries::dsl as e_dsl;
        use self::schema::entry_category_relations::dsl as e_c_dsl;
        use self::schema::entry_tag_relations::dsl as e_t_dsl;

        let entries: Vec<models::Entry> =
            e_dsl::entries.filter(e_dsl::current.eq(true)).load(self)?;

        let cat_rels =
            e_c_dsl::entry_category_relations.load::<models::EntryCategoryRelation>(self)?;

        let tag_rels = e_t_dsl::entry_tag_relations.load::<models::EntryTagRelation>(self)?;

        Ok(entries
            .into_iter()
            .map(|e| {
                let cats = cat_rels
                    .iter()
                    .filter(|r| r.entry_id == e.id)
                    .filter(|r| r.entry_version == e.version)
                    .map(|r| &r.category_id)
                    .cloned()
                    .collect();
                let tags = tag_rels
                    .iter()
                    .filter(|r| r.entry_id == e.id)
                    .filter(|r| r.entry_version == e.version)
                    .map(|r| &r.tag_id)
                    .cloned()
                    .collect();
                Entry {
                    id: e.id,
                    osm_node: e.osm_node.map(|x| x as u64),
                    created: e.created as u64,
                    version: e.version as u64,
                    title: e.title,
                    description: e.description,
                    lat: e.lat as f64,
                    lng: e.lng as f64,
                    street: e.street,
                    zip: e.zip,
                    city: e.city,
                    country: e.country,
                    email: e.email,
                    telephone: e.telephone,
                    homepage: e.homepage,
                    categories: cats,
                    tags: tags,
                    license: e.license,
                }
            })
            .collect())
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
        Ok(tags.load::<models::Tag>(self)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn all_ratings(&self) -> Result<Vec<Rating>> {
        use self::schema::ratings::dsl::*;
        Ok(ratings
            .load::<models::Rating>(self)?
            .into_iter()
            .map(Rating::from)
            .collect())
    }
    fn all_comments(&self) -> Result<Vec<Comment>> {
        use self::schema::comments::dsl::*;
        Ok(comments
            .load::<models::Comment>(self)?
            .into_iter()
            .map(Comment::from)
            .collect())
    }

    fn update_entry(&mut self, entry: &Entry) -> Result<()> {
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

        self.transaction::<_, diesel::result::Error, _>(|| {
            unset_current_on_all_entries(&self, &e.id)?;
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
        })?;
        Ok(())
    }

    fn import_multiple_entries(&mut self, new_entries: &[Entry]) -> Result<()> {
        let imports: Vec<_> = new_entries
            .into_iter()
            .map(|e| {
                let new_entry = models::Entry::from(e.clone());
                let cat_rels: Vec<_> = e.categories
                    .iter()
                    .cloned()
                    .map(|category_id| models::EntryCategoryRelation {
                        entry_id: e.id.clone(),
                        entry_version: e.version as i64,
                        category_id,
                    })
                    .collect();
                let tag_rels: Vec<_> = e.tags
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
                unset_current_on_all_entries(&self, &new_entry.id)?;
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
