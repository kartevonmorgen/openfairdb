use std::{
    net::{SocketAddr, TcpListener},
    thread,
};

use rocket::{config::Config as RocketCfg, local::blocking::Client, Route};

use crate::{
    core::{prelude::*, usecases},
    web::{api::tests::prelude::default_accepted_licenses, sqlite, tantivy, Cfg},
};

pub mod prelude {

    pub const DUMMY_VERSION: &str = "3.2.1";

    pub use rocket::{
        http::{ContentType, Cookie, Status},
        local::blocking::{Client, LocalResponse},
        response::Response,
    };

    pub use super::{run_server, DummyNotifyGW};

    pub use crate::core::db::*;
}

pub fn run_server() -> (SocketAddr, sqlite::Connections, tantivy::SearchEngine) {
    let cfg = Cfg {
        accepted_licenses: default_accepted_licenses(),
        protect_with_captcha: false,
    };

    let address = {
        let listener = TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap()).unwrap();
        listener.local_addr().unwrap()
    };

    let mut rocket_cfg = RocketCfg::debug_default();
    rocket_cfg.port = address.port();
    rocket_cfg.address = address.ip();

    let socket_address: SocketAddr = (rocket_cfg.address, rocket_cfg.port).into();

    let (rocket, db, search_engine) =
        rocket_test_instance_with_cfg(crate::web::mounts(), cfg, rocket_cfg);

    thread::spawn(move || {
        rocket::execute(rocket.launch()).unwrap();
    });

    (socket_address, db, search_engine)
}

fn rocket_test_instance_with_cfg(
    mounts: Vec<(&'static str, Vec<Route>)>,
    cfg: Cfg,
    rocket_cfg: RocketCfg,
) -> (
    rocket::Rocket<rocket::Build>,
    sqlite::Connections,
    tantivy::SearchEngine,
) {
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
    (rocket, db, search_engine)
}

pub fn rocket_test_setup(
    mounts: Vec<(&'static str, Vec<Route>)>,
) -> (Client, sqlite::Connections, tantivy::SearchEngine) {
    rocket_test_setup_with_cfg(
        mounts,
        Cfg {
            accepted_licenses: default_accepted_licenses(),
            protect_with_captcha: false,
        },
    )
}

pub fn rocket_test_setup_with_cfg(
    mounts: Vec<(&'static str, Vec<Route>)>,
    cfg: Cfg,
) -> (Client, sqlite::Connections, tantivy::SearchEngine) {
    let rocket_cfg = RocketCfg::debug_default();
    let (rocket, db, search_engine) = rocket_test_instance_with_cfg(mounts, cfg, rocket_cfg);
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
