mod add_entry;
mod add_rating;
mod archive_comments;
mod archive_entries;
mod archive_events;
mod archive_ratings;
mod update_entry;

pub mod prelude {
    pub use super::{
        add_entry::*, add_entry::*, add_rating::*, archive_comments::*, archive_entries::*,
        archive_events::*, archive_ratings::*, update_entry::*, Result,
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
            ports::web::api
        };

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
                let rocket =
                    rocket_instance(db_connections.clone(), search_engine.clone(), vec![("/", api::routes())], Some(cfg));
                let client = Client::new(rocket).unwrap();
                Self {
                    client,
                    db_connections,
                    search_engine: RefCell::new(search_engine),
                }
            }

            pub fn add_entry(self: &EnvFixture, new_entry: NewEntry) -> String {
                flows::add_entry(
                    &self.db_connections,
                    &mut *self.search_engine.borrow_mut(),
                    new_entry.into(),
                )
                .unwrap()
                .id
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

            pub fn query_entries(self: &EnvFixture, query: &EntryIndexQuery) -> Vec<IndexedEntry> {
                self.search_engine
                    .borrow_mut()
                    .query_entries(query, 100)
                    .unwrap()
            }

            pub fn query_entries_by_tag(self: &EnvFixture, tag: &str) -> Vec<IndexedEntry> {
                let query = EntryIndexQuery {
                    tags: vec![tag.into()],
                    ..Default::default()
                };
                self.query_entries(&query)
            }
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
    }
}
