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

    use crate::ports::web::tests::DummyNotifyGW;

    use std::cell::RefCell;

    embed_migrations!();

    pub struct BackendFixture {
        pub db_connections: sqlite::Connections,
        pub search_engine: RefCell<tantivy::SearchEngine>,
        pub notify: DummyNotifyGW,
    }

    impl BackendFixture {
        pub fn new() -> Self {
            let db_connections = sqlite::Connections::init(":memory:", 1).unwrap();
            embedded_migrations::run(&*db_connections.exclusive().unwrap()).unwrap();
            let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
            Self {
                db_connections,
                search_engine: RefCell::new(search_engine),
                notify: DummyNotifyGW,
            }
        }

        pub fn create_place(&self, new_place: NewPlace, account_email: Option<&str>) -> String {
            flows::create_place(
                &self.db_connections,
                &mut *self.search_engine.borrow_mut(),
                &self.notify,
                new_place.into(),
                account_email,
                None,
            )
            .unwrap()
            .id
            .into()
        }

        pub fn create_user(&self, new_user: usecases::NewUser, role: Option<Role>) {
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

        pub fn try_get_user(&self, email: &str) -> Option<User> {
            self.db_connections
                .shared()
                .unwrap()
                .try_get_user_by_email(email)
                .unwrap_or_default()
        }

        pub fn try_get_place(&self, id: &str) -> Option<(Place, ReviewStatus)> {
            match self.db_connections.shared().unwrap().get_place(id) {
                Ok(x) => Some(x),
                Err(RepoError::NotFound) => None,
                x => x.map(|_| None).unwrap(),
            }
        }

        pub fn place_exists(&self, id: &str) -> bool {
            self.try_get_place(id).filter(|(_, s)| s.exists()).is_some()
        }

        pub fn create_rating(&self, rate_entry: usecases::NewPlaceRating) -> (String, String) {
            flows::create_rating(
                &self.db_connections,
                &mut *self.search_engine.borrow_mut(),
                rate_entry,
            )
            .unwrap()
        }

        pub fn try_get_rating(&self, id: &str) -> Option<Rating> {
            match self.db_connections.shared().unwrap().load_rating(id) {
                Ok(rating) => Some(rating),
                Err(RepoError::NotFound) => None,
                x => x.map(|_| None).unwrap(),
            }
        }

        pub fn rating_exists(&self, id: &str) -> bool {
            self.try_get_rating(id).is_some()
        }

        pub fn try_get_comment(&self, id: &str) -> Option<Comment> {
            match self.db_connections.shared().unwrap().load_comment(id) {
                Ok(comment) => Some(comment),
                Err(RepoError::NotFound) => None,
                x => x.map(|_| None).unwrap(),
            }
        }

        pub fn comment_exists(&self, id: &str) -> bool {
            self.try_get_comment(id).is_some()
        }

        pub fn query_places(&self, query: &IndexQuery) -> Vec<IndexedPlace> {
            self.search_engine
                .borrow_mut()
                .query_places(query, 100)
                .unwrap()
        }

        pub fn query_places_by_tag(&self, tag: &str) -> Vec<IndexedPlace> {
            let query = IndexQuery {
                hash_tags: vec![tag.into()],
                status: Some(vec![]), // only visible/existent places
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
                state: None,
                email: None,
                telephone: None,
                homepage: None,
                opening_hours: None,
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
        let context = context.into();
        let value = value.into();
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
