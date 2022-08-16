#![allow(proc_macro_derive_resolution_fallback)]
#![recursion_limit = "128"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use ofdb_core::gateways::{geocode::GeoCodingGateway, notify::NotificationGateway};
use ofdb_db_sqlite::Connections;
use ofdb_db_tantivy as tantivy;

mod adapters;
mod core;
mod web;

pub use web::Cfg;

pub async fn run(
    connections: Connections,
    search_engine: tantivy::SearchEngine,
    enable_cors: bool,
    cfg: Cfg,
    geo_gw: Box<dyn GeoCodingGateway + Send + Sync>,
    notify_gw: Box<dyn NotificationGateway + Send + Sync>,
) {
    let search_engine = web::tantivy::SearchEngine(search_engine);

    web::run(
        connections.into(),
        search_engine,
        enable_cors,
        cfg,
        geo_gw,
        notify_gw,
    )
    .await;
}
