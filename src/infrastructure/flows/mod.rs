mod archive_comments;
mod archive_events;
mod archive_ratings;
mod change_user_role;
mod create_place;
mod create_rating;
mod reset_password;
mod review_places;
mod update_place;

pub mod prelude {
    pub use super::{
        archive_comments::*, archive_events::*, archive_ratings::*, change_user_role::*,
        create_place::*, create_rating::*, reset_password::*, review_places::*, update_place::*,
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

        pub use rocket::{
            http::{ContentType, Cookie, Status as HttpStatus},
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
                let db_connections = sqlite::Connections::init(":memory:", 1).unwrap();
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

            pub fn create_place(
                self: &EnvFixture,
                new_place: NewPlace,
                account_email: Option<&str>,
            ) -> String {
                flows::create_place(
                    &self.db_connections,
                    &mut *self.search_engine.borrow_mut(),
                    new_place.into(),
                    account_email,
                )
                .unwrap()
                .id
                .into()
            }

            pub fn create_user(self: &EnvFixture, new_user: usecases::NewUser, role: Option<Role>) {
                let email = {
                    let db = self.db_connections.exclusive().unwrap();
                    let email = new_user.email.clone();
                    usecases::create_new_user(&*db, new_user).unwrap();
                    email
                };
                if let Some(role) = role {
                    let mut u = self.try_get_user(&email).unwrap();
                    u.role = role;
                    let db = self.db_connections.exclusive().unwrap();
                    db.update_user(&u).unwrap();
                }
            }

            pub fn try_get_user(self: &EnvFixture, email: &str) -> Option<User> {
                self.db_connections
                    .shared()
                    .unwrap()
                    .try_get_user_by_email(email)
                    .unwrap_or_default()
            }

            pub fn try_get_place(self: &EnvFixture, id: &str) -> Option<(Place, ReviewStatus)> {
                match self.db_connections.shared().unwrap().get_place(id) {
                    Ok(x) => Some(x),
                    Err(RepoError::NotFound) => None,
                    x => x.map(|_| None).unwrap(),
                }
            }

            pub fn place_exists(self: &EnvFixture, id: &str) -> bool {
                self.try_get_place(id).filter(|(_, s)| s.exists()).is_some()
            }

            pub fn create_rating(
                self: &EnvFixture,
                rate_entry: usecases::NewPlaceRating,
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

            pub fn query_places(self: &EnvFixture, query: &IndexQuery) -> Vec<IndexedPlace> {
                self.search_engine
                    .borrow_mut()
                    .query_places(query, 100)
                    .unwrap()
            }

            pub fn query_places_by_tag(self: &EnvFixture, tag: &str) -> Vec<IndexedPlace> {
                let query = IndexQuery {
                    hash_tags: vec![tag.into()],
                    ..Default::default()
                };
                self.query_places(&query)
            }
        }

        pub struct NewPlace {
            pub pos: MapPoint,
            pub title: String,
            pub description: String,
            pub categories: Vec<String>,
            pub tags: Vec<String>,
        }

        impl From<i32> for NewPlace {
            fn from(i: i32) -> Self {
                let lat_deg = i % 91;
                let lng_deg = -i % 181;
                let pos = MapPoint::from_lat_lng_deg(f64::from(lat_deg), f64::from(lng_deg));
                let title = format!("title_{}", i);
                let description = format!("description_{}", i);
                let categories = vec![format!("category_{}", i)];
                let tags = vec![format!("tag_{}", i)];
                NewPlace {
                    pos,
                    title,
                    description,
                    categories,
                    tags,
                }
            }
        }

        impl From<NewPlace> for usecases::NewPlace {
            fn from(e: NewPlace) -> Self {
                usecases::NewPlace {
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
            place_id: &str,
            context: RatingContext,
            value: RatingValue,
        ) -> usecases::NewPlaceRating {
            usecases::NewPlaceRating {
                entry: place_id.to_owned(),
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
