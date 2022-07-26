use std::{env, path::Path};

use clap::{crate_authors, Arg, Command};
use dotenv::dotenv;
use ofdb_core::gateways::geocode::GeoCodingGateway;

use crate::{
    core::prelude::*,
    infrastructure::{cfg::Cfg, db::tantivy, GEO_CODING_GW},
    ports::web,
};
use ofdb_db_sqlite::Connections;

fn update_event_locations<R>(repo: &R) -> Result<()>
where
    R: EventRepo,
{
    let events = repo.all_events_chronologically()?;
    for mut e in events {
        if let Some(ref mut loc) = e.location {
            if let Some(ref addr) = loc.address {
                if let Some((lat, lng)) = GEO_CODING_GW.resolve_address_lat_lng(addr) {
                    if let Ok(pos) = MapPoint::try_from_lat_lng_deg(lat, lng) {
                        if pos.is_valid() {
                            if let Err(err) = repo.update_event(&e) {
                                warn!("Failed to update location of event {}: {}", e.id, err);
                            } else {
                                info!("Updated location of event {}", e.id);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[allow(deprecated)]
pub fn run() {
    dotenv().ok(); // TODO: either use environment variables XOR cli arguments
    let matches = Command::new("openFairDB")
        .version(env!("CARGO_PKG_VERSION"))
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("db-url")
                .long("db-url")
                .value_name("DATABASE_URL")
                .help("URL to the database"),
        )
        .arg(
            Arg::with_name("idx-dir")
                .long("idx-dir")
                .value_name("INDEX_DIR")
                .help("File system directory for the full-text search index"),
        )
        .arg(
            Arg::with_name("enable-cors")
                .long("enable-cors")
                .help("Allow requests from any origin"),
        )
        .arg(
            Arg::with_name("fix-event-address-location")
                .long("fix-event-address-location")
                .help("Update the location of ALL events by resolving their address"),
        )
        .get_matches();

    let mut cfg = Cfg::from_env_or_default();

    if let Some(db_url) = matches.value_of("db-url").map(ToString::to_string) {
        cfg.db_url = db_url
    }
    info!(
        "Connecting to SQLite database '{}' (pool size = {})",
        cfg.db_url, cfg.db_connection_pool_size
    );
    let connections = Connections::init(&cfg.db_url, cfg.db_connection_pool_size).unwrap();

    ofdb_db_sqlite::run_embedded_database_migrations(connections.exclusive().unwrap());

    let idx_dir = matches
        .value_of("idx-dir")
        .map(ToString::to_string)
        .or_else(|| env::var("INDEX_DIR").map(Option::Some).unwrap_or(None));
    let idx_path = idx_dir.as_ref().map(Path::new);
    info!("Initializing Tantivy full-text search engine");
    let search_engine = tantivy::SearchEngine::init_with_path(idx_path).unwrap();

    #[allow(clippy::match_single_binding)]
    match matches.subcommand() {
        _ => {
            if matches.is_present("fix-event-address-location") {
                info!("Updating all event locations...");
                update_event_locations(&connections.exclusive().unwrap()).unwrap();
            }
            web::run(
                connections.into(),
                search_engine,
                matches.is_present("enable-cors"),
                cfg,
            );
        }
    }
}
