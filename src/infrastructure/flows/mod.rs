mod archive_comments;
mod archive_entries;
mod archive_events;
mod archive_ratings;
mod create_entry;
mod create_rating;
mod update_entry;

pub mod prelude {
    pub use super::{
        archive_comments::*, archive_entries::*, archive_events::*, archive_ratings::*,
        create_entry::*, create_rating::*, update_entry::*,
    };
}

pub type Result<T> = std::result::Result<T, error::AppError>;

pub(crate) use super::{db::sqlite, error, notify};

pub(crate) use crate::core::{prelude::*, usecases};

#[cfg(test)]
mod tests {
    pub mod prelude {
        pub use crate::core::{prelude::*, usecases};
        pub mod sqlite {
            pub use super::super::super::sqlite::*;
        }
        pub mod tantivy {
            pub use crate::infrastructure::db::tantivy::SearchEngine;
        }
        pub use crate::{
            infrastructure::{error::AppError, flows::prelude as flows},
            ports::web::api,
        };

        use super::super::Result;

        pub use rocket::{
            http::{ContentType, Cookie, Status},
            local::Client,
            response::Response,
        };

        use crate::ports::web::rocket_instance;

        use rocket::{
            config::{Config, Environment},
            logger::LoggingLevel,
        };
        use std::cell::RefCell;

        embed_migrations!();

        pub struct EnvFixture {
            pub client: Client,
            pub db_connections: sqlite::Connections,
            pub search_engine: RefCell<tantivy::SearchEngine>,
        }

        impl EnvFixture {
            pub fn new() -> Self {
                let cfg = Config::build(Environment::Development)
                    .log_level(LoggingLevel::Debug)
                    .finalize()
                    .unwrap();
                let db_connections = sqlite::Connections::init(&format!(":memory:"), 1).unwrap();
                embedded_migrations::run(&*db_connections.exclusive().unwrap()).unwrap();
                let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
                let rocket = rocket_instance(
                    db_connections.clone(),
                    search_engine.clone(),
                    vec![("/", api::routes())],
                    Some(cfg),
                );
                let client = Client::new(rocket).unwrap();
                Self {
                    client,
                    db_connections,
                    search_engine: RefCell::new(search_engine),
                }
            }

            pub fn create_entry(self: &EnvFixture, new_entry: NewEntry) -> String {
                flows::create_entry(
                    &self.db_connections,
                    &mut *self.search_engine.borrow_mut(),
                    new_entry.into(),
                )
                .unwrap()
            }

            pub fn try_get_entry(self: &EnvFixture, id: &str) -> Option<Entry> {
                match self.db_connections.shared().unwrap().get_entry(id) {
                    Ok(entry) => Some(entry),
                    Err(RepoError::NotFound) => None,
                    x => x.map(|_| None).unwrap(),
                }
            }

            pub fn entry_exists(self: &EnvFixture, id: &str) -> bool {
                self.try_get_entry(id).is_some()
            }

            pub fn create_rating(
                self: &EnvFixture,
                rate_entry: usecases::RateEntry,
            ) -> (String, String) {
                flows::create_rating(
                    &self.db_connections,
                    &mut *self.search_engine.borrow_mut(),
                    rate_entry,
                )
                .unwrap()
            }

            pub fn try_get_rating(self: &EnvFixture, id: &str) -> Option<Rating> {
                match self.db_connections.shared().unwrap().load_rating(id) {
                    Ok(rating) => Some(rating),
                    Err(RepoError::NotFound) => None,
                    x => x.map(|_| None).unwrap(),
                }
            }

            pub fn rating_exists(self: &EnvFixture, id: &str) -> bool {
                self.try_get_rating(id).is_some()
            }

            pub fn try_get_comment(self: &EnvFixture, id: &str) -> Option<Comment> {
                match self.db_connections.shared().unwrap().load_comment(id) {
                    Ok(comment) => Some(comment),
                    Err(RepoError::NotFound) => None,
                    x => x.map(|_| None).unwrap(),
                }
            }

            pub fn comment_exists(self: &EnvFixture, id: &str) -> bool {
                self.try_get_comment(id).is_some()
            }

            pub fn query_entries(self: &EnvFixture, query: &EntryIndexQuery) -> Vec<IndexedEntry> {
                self.search_engine
                    .borrow_mut()
                    .query_entries(query, 100)
                    .unwrap()
            }

            pub fn query_entries_by_tag(self: &EnvFixture, tag: &str) -> Vec<IndexedEntry> {
                let query = EntryIndexQuery {
                    hash_tags: vec![tag.into()],
                    ..Default::default()
                };
                self.query_entries(&query)
            }
        }

        pub fn assert_not_found<T: std::fmt::Debug>(res: Result<T>) {
            assert_eq!(
                RepoError::NotFound.to_string(),
                res.unwrap_err().to_string()
            );
        }

        pub struct NewEntry {
            pub pos: MapPoint,
            pub title: String,
            pub description: String,
            pub categories: Vec<String>,
            pub tags: Vec<String>,
        }

        impl From<i32> for NewEntry {
            fn from(i: i32) -> Self {
                let lat_deg = i % 91;
                let lng_deg = -i % 181;
                let pos = MapPoint::from_lat_lng_deg(lat_deg as f64, lng_deg as f64);
                let title = format!("title_{}", i);
                let description = format!("description_{}", i);
                let categories = vec![format!("category_{}", i)];
                let tags = vec![format!("tag_{}", i)];
                NewEntry {
                    pos,
                    title,
                    description,
                    categories,
                    tags,
                }
            }
        }

        impl From<NewEntry> for usecases::NewEntry {
            fn from(e: NewEntry) -> Self {
                usecases::NewEntry {
                    lat: e.pos.lat().to_deg(),
                    lng: e.pos.lng().to_deg(),
                    title: e.title,
                    description: e.description,
                    categories: e.categories,
                    tags: e.tags,
                    license: "CC0-1.0".into(),
                    street: None,
                    city: None,
                    zip: None,
                    country: None,
                    email: None,
                    telephone: None,
                    homepage: None,
                    image_url: None,
                    image_link_url: None,
                }
            }
        }

        pub fn new_entry_rating(
            i: i32,
            entry_id: &str,
            context: RatingContext,
            value: RatingValue,
        ) -> usecases::RateEntry {
            usecases::RateEntry {
                entry: entry_id.to_owned(),
                context,
                value,
                title: format!("title_{}", i),
                comment: format!("comment_{}", i),
                source: None,
                user: None,
            }
        }
    }
}
