use rocket::{config::Config as RocketCfg, local::blocking::Client, Route};

use crate::{
    core::{prelude::*, usecases},
    web::{sqlite, tantivy, Cfg},
};

pub mod prelude {

    pub const DUMMY_VERSION: &str = "3.2.1";

    pub use rocket::{
        http::{ContentType, Cookie, Status},
        local::blocking::{Client, LocalResponse},
        response::Response,
    };

    pub use super::DummyNotifyGW;
    pub use crate::core::db::*;
}

pub fn setup(
    mounts: Vec<(&'static str, Vec<Route>)>,
) -> (Client, sqlite::Connections, tantivy::SearchEngine) {
    setup_with_cfg(
        mounts,
        Cfg {
            accepted_licenses: crate::web::api::tests::prelude::default_accepted_licenses(),
            protect_with_captcha: false,
        },
    )
}

pub fn setup_with_cfg(
    mounts: Vec<(&'static str, Vec<Route>)>,
    cfg: Cfg,
) -> (Client, sqlite::Connections, tantivy::SearchEngine) {
    let rocket_cfg = RocketCfg::debug_default();
    let connections = ofdb_db_sqlite::Connections::init(":memory:", 1).unwrap();
    ofdb_db_sqlite::run_embedded_database_migrations(connections.exclusive().unwrap());
    let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
    let db = sqlite::Connections::from(connections);
    let notify_gw = DummyNotifyGW;
    let geo_gw = DummyGeoGW;
    let connections = super::Connections {
        db: db.clone(),
        search_engine: search_engine.clone(),
    };
    let options = super::InstanceOptions {
        mounts,
        rocket_cfg: Some(rocket_cfg),
        cfg,
        version: prelude::DUMMY_VERSION,
    };

    let gateways = super::Gateways {
        geocoding: Box::new(geo_gw),
        notify: Box::new(notify_gw),
    };
    let rocket = super::rocket_instance(options, connections, gateways);
    let client = Client::tracked(rocket).unwrap();
    (client, db, search_engine)
}

pub fn register_user(pool: &sqlite::Connections, email: &str, pw: &str, confirmed: bool) {
    let email = email.parse::<EmailAddress>().unwrap();
    let db = pool.exclusive().unwrap();
    usecases::create_new_user(
        &db,
        usecases::NewUser {
            email: email.clone(),
            password: pw.to_string(),
        },
    )
    .unwrap();
    let email_nonce = EmailNonce {
        email,
        nonce: Nonce::new(),
    };
    let token = email_nonce.encode_to_string();
    if confirmed {
        usecases::confirm_email_address(&db, &token).unwrap();
    }
}

pub struct DummyNotifyGW;

use ofdb_core::gateways::notify::{NotificationEvent, NotificationGateway};

impl NotificationGateway for DummyNotifyGW {
    fn notify(&self, _: NotificationEvent) {}
}

pub struct DummyGeoGW;

impl ofdb_core::gateways::geocode::GeoCodingGateway for DummyGeoGW {
    fn resolve_address_lat_lng(&self, _: &ofdb_core::entities::Address) -> Option<(f64, f64)> {
        None
    }
}
