use crate::{
    core::{prelude::*, usecases},
    infrastructure::db::{sqlite, tantivy},
};
use rocket::{
    config::{Config, Environment},
    local::Client,
    logger::LoggingLevel,
    Route,
};

pub mod prelude {
    pub use super::DummyNotifyGW;
    pub use crate::core::db::*;
    pub use rocket::{
        http::{ContentType, Cookie, Status},
        local::Client,
        response::Response,
    };
}

embed_migrations!();

pub fn setup(
    mounts: Vec<(&'static str, Vec<Route>)>,
) -> (
    rocket::local::Client,
    sqlite::Connections,
    tantivy::SearchEngine,
) {
    let cfg = Config::build(Environment::Development)
        .log_level(LoggingLevel::Debug)
        .finalize()
        .unwrap();
    let connections = sqlite::Connections::init(":memory:", 1).unwrap();
    embedded_migrations::run(&*connections.exclusive().unwrap()).unwrap();
    let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
    let rocket = super::rocket_instance(
        connections.clone(),
        search_engine.clone(),
        mounts,
        Some(cfg),
    );
    let client = Client::new(rocket).unwrap();
    (client, connections, search_engine)
}

pub fn register_user(pool: &sqlite::Connections, email: &str, pw: &str, confirmed: bool) {
    let db = pool.exclusive().unwrap();
    usecases::create_new_user(
        &*db,
        usecases::NewUser {
            email: email.to_string(),
            password: pw.to_string(),
        },
    )
    .unwrap();
    let email_nonce = EmailNonce {
        email: email.to_string(),
        nonce: Nonce::new(),
    };
    let token = email_nonce.encode_to_string();
    if confirmed {
        usecases::confirm_email_address(&*db, &token).unwrap();
    }
}

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
