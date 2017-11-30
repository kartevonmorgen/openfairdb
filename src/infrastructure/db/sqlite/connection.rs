use entities::*;
use business::error::RepoError;
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::result;
use business::db::Db;
use super::models;
use super::schema;

type Result<T> = result::Result<T, RepoError>;

impl Db for SqliteConnection {
    fn create_entry(&mut self, e: &Entry) -> Result<()> {
        let new_entry = models::Entry::from(e.clone());
        let cat_rels: Vec<_> = e.categories
            .iter()
            .cloned()
            .map(|category_id| {
                models::EntryCategoryRelation {
                    entry_id: e.id.clone(),
                    category_id,
                }
            })
            .collect();
        diesel::insert_into(schema::entries::table)
            .values(&new_entry)
            .execute(self)?;
        diesel::insert_into(schema::entry_category_relations::table)
            //WHERE NOT EXISTS
            .values(&cat_rels)
            .execute(self)?;
        Ok(())
    }
    fn create_tag(&mut self, t: &Tag) -> Result<()> {
        diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self)?;
        Ok(())
    }
    fn create_triple(&mut self, t: &Triple) -> Result<()> {
        diesel::insert_into(schema::triples::table)
            .values(&models::Triple::from(t.clone()))
            .execute(self)?;
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
        Ok(
            dsl::users
                .load::<models::User>(self)?
                .into_iter()
                .map(User::from)
                .collect(),
        )
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        use self::schema::bbox_subscriptions::dsl;
        Ok(
            dsl::bbox_subscriptions
                .load::<models::BboxSubscription>(self)?
                .into_iter()
                .map(BboxSubscription::from)
                .collect(),
        )
    }
    fn confirm_email_address(&mut self, username: &str) -> Result<User> {
        use self::schema::users::dsl;
        diesel::update(dsl::users.find(username))
            .set(dsl::email_confirmed.eq(true))
            .execute(self)?;
        Ok(User::from(self.get_user(username)?))
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

        let models::Entry {
            id,
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
        } = e_dsl::entries.find(e_id).first(self)?;

        let categories = e_c_dsl::entry_category_relations
            .filter(e_c_dsl::entry_id.eq(&id))
            .load::<models::EntryCategoryRelation>(self)?
            .into_iter()
            .map(|r| r.category_id)
            .collect();

        Ok(Entry {
            id,
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
            license,
        })
    }

    fn get_user(&self, user_id: &str) -> Result<User> {
        use self::schema::users::dsl::*;
        let u: models::User = users.find(user_id).first(self)?;
        Ok(User::from(u))
    }

    fn all_entries(&self) -> Result<Vec<Entry>> {
        use self::schema::entries::dsl as e_dsl;
        use self::schema::entry_category_relations::dsl as e_c_dsl;

        let entries: Vec<models::Entry> = e_dsl::entries.load(self)?;
        let cat_rels = e_c_dsl::entry_category_relations
            .load::<models::EntryCategoryRelation>(self)?;

        Ok(
            entries
                .into_iter()
                .map(|e| {
                    let cats = cat_rels
                        .iter()
                        .filter(|r| r.entry_id == e.id)
                        .map(|r| &r.category_id)
                        .cloned()
                        .collect();
                    Entry {
                        id: e.id,
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
                        license: e.license,
                    }
                })
                .collect(),
        )
    }
    fn all_categories(&self) -> Result<Vec<Category>> {
        use self::schema::categories::dsl::*;
        Ok(
            categories
                .load::<models::Category>(self)?
                .into_iter()
                .map(Category::from)
                .collect(),
        )
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use self::schema::tags::dsl::*;
        Ok(
            tags.load::<models::Tag>(self)?
                .into_iter()
                .map(Tag::from)
                .collect(),
        )

    }
    fn all_triples(&self) -> Result<Vec<Triple>> {
        use self::schema::triples::dsl::*;
        Ok(
            triples
                .load::<models::Triple>(self)?
                .into_iter()
                .map(Triple::from)
                .collect(),
        )
    }
    fn all_ratings(&self) -> Result<Vec<Rating>> {
        use self::schema::ratings::dsl::*;
        Ok(
            ratings
                .load::<models::Rating>(self)?
                .into_iter()
                .map(Rating::from)
                .collect(),
        )
    }
    fn all_comments(&self) -> Result<Vec<Comment>> {
        use self::schema::comments::dsl::*;
        Ok(
            comments
                .load::<models::Comment>(self)?
                .into_iter()
                .map(Comment::from)
                .collect(),
        )
    }

    fn update_entry(&mut self, entry: &Entry) -> Result<()> {

        use self::schema::entries::dsl as e_dsl;
        use self::schema::entry_category_relations::dsl as e_c_dsl;

        let e = models::Entry::from(entry.clone());

        diesel::update(e_dsl::entries.find(e.id))
            .set((
                e_dsl::created.eq(e.created),
                e_dsl::version.eq(e.version),
                e_dsl::title.eq(e.title),
                e_dsl::description.eq(e.description),
                e_dsl::lat.eq(e.lat),
                e_dsl::lng.eq(e.lng),
                e_dsl::street.eq(e.street),
                e_dsl::zip.eq(e.zip),
                e_dsl::city.eq(e.city),
                e_dsl::country.eq(e.country),
                e_dsl::email.eq(e.email),
                e_dsl::telephone.eq(e.telephone),
                e_dsl::homepage.eq(e.homepage),
                e_dsl::license.eq(e.license),
            ))
            .execute(self)?;

        diesel::delete(e_c_dsl::entry_category_relations.filter(
            e_c_dsl::entry_id.eq(
                &entry.id,
            ),
        )).execute(self)?;

        let cat_rels: Vec<_> = entry
            .categories
            .iter()
            .cloned()
            .map(|category_id| {
                models::EntryCategoryRelation {
                    entry_id: entry.id.clone(),
                    category_id,
                }
            })
            .collect();

        diesel::insert_into(schema::entry_category_relations::table)
            //WHERE NOT EXISTS
            .values(&cat_rels)
            .execute(self)?;

        Ok(())
    }

    fn delete_triple(&mut self, t: &Triple) -> Result<()> {
        use self::schema::triples::dsl::*;
        let t = models::Triple::from(t.clone());
        diesel::delete(triples.find((t.subject_id, t.predicate, t.object_id)))
            .execute(self)?;
        Ok(())
    }
}
