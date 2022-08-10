mod clearance;
mod search;

pub mod prelude {

    pub fn accepted_licenses() -> HashSet<String> {
        let mut licenses = HashSet::new();
        licenses.insert("CC0-1.0".into());
        licenses.insert("ODbL-1.0".into());
        licenses
    }

    pub fn default_new_place() -> usecases::NewPlace {
        usecases::NewPlace {
            title: Default::default(),
            description: Default::default(),
            categories: Default::default(),
            contact_name: None,
            email: None,
            telephone: None,
            lat: Default::default(),
            lng: Default::default(),
            street: None,
            zip: None,
            city: None,
            country: None,
            state: None,
            tags: vec![],
            homepage: None,
            opening_hours: None,
            founded_on: None,
            license: "CC0-1.0".into(),
            image_url: None,
            image_link_url: None,
            custom_links: vec![],
        }
    }

    pub fn default_search_request<'a>() -> usecases::SearchRequest<'a> {
        usecases::SearchRequest {
            bbox: MapBbox::new(
                MapPoint::from_lat_lng_deg(-90, -180),
                MapPoint::from_lat_lng_deg(90, 180),
            ),
            org_tag: None,
            categories: vec![],
            hash_tags: vec![],
            ids: vec![],
            status: vec![],
            text: None,
        }
    }

    use std::{cell::RefCell, collections::HashSet};

    pub use ofdb_core::{
        db::*,
        entities::*,
        repositories::{Error as RepoError, *},
        usecases,
    };

    pub mod sqlite {
        pub use super::super::super::sqlite::*;
    }

    pub mod tantivy {
        pub use ofdb_db_tantivy::SearchEngine;
    }

    pub use crate::{error::AppError, prelude as flows};

    pub struct DummyNotifyGW;

    impl ofdb_core::gateways::notify::NotificationGateway for DummyNotifyGW {
        fn place_added(&self, _: &[String], _: &Place, _: Vec<Category>) {}
        fn place_updated(&self, _: &[String], _: &Place, _: Vec<Category>) {}
        fn event_created(&self, _: &[String], _: &Event) {}
        fn event_updated(&self, _: &[String], _: &Event) {}
        fn user_registered_kvm(&self, _: &User) {}
        fn user_registered_ofdb(&self, _: &User) {}
        fn user_registered(&self, _: &User, _: &str) {}
        fn user_reset_password_requested(&self, _: &EmailNonce) {}
    }

    pub struct BackendFixture {
        pub db_connections: sqlite::Connections,
        pub search_engine: RefCell<tantivy::SearchEngine>,
        pub notify: DummyNotifyGW,
    }

    impl BackendFixture {
        pub fn new() -> Self {
            let db_connections = sqlite::Connections::init(":memory:", 1).unwrap();
            ofdb_db_sqlite::run_embedded_database_migrations(db_connections.exclusive().unwrap());
            let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
            Self {
                db_connections,
                search_engine: RefCell::new(search_engine),
                notify: DummyNotifyGW,
            }
        }

        pub fn create_place(&self, new_place: NewPlace, account_email: Option<&str>) -> String {
            let mut accepted_licenses = HashSet::new();
            accepted_licenses.insert("CC0-1.0".into());
            accepted_licenses.insert("ODbL-1.0".into());
            flows::create_place(
                &self.db_connections,
                &mut *self.search_engine.borrow_mut(),
                &self.notify,
                new_place.into(),
                account_email,
                None,
                &accepted_licenses,
            )
            .unwrap()
            .id
            .into()
        }

        pub fn create_user(&self, new_user: usecases::NewUser, role: Option<Role>) {
            let email = {
                let db = self.db_connections.exclusive().unwrap();
                let email = new_user.email.clone();
                usecases::create_new_user(&db, new_user).unwrap();
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
        pub custom_links: Vec<CustomLink>,
    }

    impl From<i32> for NewPlace {
        fn from(i: i32) -> Self {
            let lat_deg = i % 91;
            let lng_deg = -i % 181;
            let pos = MapPoint::from_lat_lng_deg(f64::from(lat_deg), f64::from(lng_deg));
            let title = format!("Title {}", i);
            let description = format!("Description {}", i);
            let categories = vec![format!("category_{}", i)];
            let tags = vec![format!("tag-{}", i)];
            let custom_links = vec![CustomLink {
                url: format!("https://www.example{}.com", i).parse().unwrap(),
                title: Some(format!("Custom link {}", i)),
                description: None,
            }];
            NewPlace {
                pos,
                title,
                description,
                categories,
                tags,
                custom_links,
            }
        }
    }

    impl From<NewPlace> for usecases::NewPlace {
        fn from(e: NewPlace) -> Self {
            let NewPlace {
                categories,
                custom_links,
                description,
                pos,
                tags,
                title,
            } = e;
            usecases::NewPlace {
                lat: pos.lat().to_deg(),
                lng: pos.lng().to_deg(),
                title,
                description,
                categories,
                tags,
                license: "CC0-1.0".into(),
                street: None,
                city: None,
                zip: None,
                country: None,
                state: None,
                contact_name: None,
                email: None,
                telephone: None,
                homepage: None,
                opening_hours: None,
                founded_on: None,
                image_url: None,
                image_link_url: None,
                custom_links: custom_links.into_iter().map(Into::into).collect(),
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
            title: format!("Rating title {}", i),
            comment: format!("Rating comment {}", i),
            source: None,
            user: None,
        }
    }
}
