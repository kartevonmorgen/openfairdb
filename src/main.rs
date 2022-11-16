// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>
// Copyright (c) 2018 - 2022 slowtec GmbH <post@slowtec.de>

use std::{env, path::Path, time::Duration};

use clap::{crate_authors, Arg, ArgAction, Command};
use dotenv::dotenv;

use ofdb_core::{
    entities::MapPoint, gateways::geocode::GeoCodingGateway, repositories::EventRepo, RepoError,
};

use ofdb_db_sqlite::Connections;
use ofdb_db_tantivy as tantivy;

mod cfg;
mod gateways;
mod recurring_reminder;

const DATABASE_URL_ARG: &str = "db-url";

const INDEX_DIR_ARG: &str = "idx-dir";

const FIX_EVENT_ADDRESS_LOCATION_ARG: &str = "fix-event-address-location";

const ENABLE_CORS_ARG: &str = "enable-cors";

fn update_event_locations<R, G>(repo: &R, geo: &G) -> Result<(), RepoError>
where
    R: EventRepo,
    G: GeoCodingGateway,
{
    let events = repo.all_events_chronologically()?;
    for mut e in events {
        if let Some(ref mut loc) = e.location {
            if let Some(ref addr) = loc.address {
                if let Some((lat, lng)) = geo.resolve_address_lat_lng(addr) {
                    if let Ok(pos) = MapPoint::try_from_lat_lng_deg(lat, lng) {
                        if pos.is_valid() {
                            if let Err(err) = repo.update_event(&e) {
                                log::warn!("Failed to update location of event {}: {}", e.id, err);
                            } else {
                                log::info!("Updated location of event {}", e.id);
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
#[tokio::main]
pub async fn main() {
    env_logger::init();
    dotenv().ok(); // TODO: either use environment variables XOR cli arguments
    let matches = Command::new("openFairDB")
        .version(env!("CARGO_PKG_VERSION"))
        .author(crate_authors!("\n"))
        .arg(
            Arg::new(DATABASE_URL_ARG)
                .long(DATABASE_URL_ARG)
                .value_name("DATABASE_URL")
                .help("URL to the database"),
        )
        .arg(
            Arg::new(INDEX_DIR_ARG)
                .long(INDEX_DIR_ARG)
                .value_name("INDEX_DIR")
                .help("File system directory for the full-text search index"),
        )
        .arg(
            Arg::new(ENABLE_CORS_ARG)
                .long(ENABLE_CORS_ARG)
                .action(ArgAction::SetTrue)
                .help("Allow requests from any origin"),
        )
        .arg(
            Arg::new(FIX_EVENT_ADDRESS_LOCATION_ARG)
                .long(FIX_EVENT_ADDRESS_LOCATION_ARG)
                .action(ArgAction::SetTrue)
                .help("Update the location of ALL events by resolving their address"),
        )
        .get_matches();

    let mut cfg = cfg::Cfg::from_env_or_default();

    if let Some(db_url) = matches.get_one::<String>(DATABASE_URL_ARG).cloned() {
        cfg.db_url = db_url;
    }
    log::info!(
        "Connecting to SQLite database '{}' (pool size = {})",
        cfg.db_url,
        cfg.db_connection_pool_size
    );
    let connections = Connections::init(&cfg.db_url, cfg.db_connection_pool_size).unwrap();

    ofdb_db_sqlite::run_embedded_database_migrations(connections.exclusive().unwrap());

    let idx_dir = matches
        .get_one::<String>(INDEX_DIR_ARG)
        .cloned()
        .or_else(|| env::var("INDEX_DIR").map(Option::Some).unwrap_or(None));
    let idx_path = idx_dir.as_ref().map(Path::new);
    log::info!("Initializing Tantivy full-text search engine");
    let search_engine = tantivy::SearchEngine::init_with_path(idx_path).unwrap();

    let geo_gw = gateways::geocoding_gateway(&cfg);
    let notify_gw = gateways::notification_gateway();

    let db_connections = connections.clone();

    tokio::spawn(async move {
        let task_interval = Duration::from_secs(60 * 60 * 24);
        recurring_reminder::run(&db_connections, task_interval).await;
    });

    #[allow(clippy::match_single_binding)]
    match matches.subcommand() {
        _ => {
            if matches.get_flag(FIX_EVENT_ADDRESS_LOCATION_ARG) {
                log::info!("Updating all event locations...");
                update_event_locations(&connections.exclusive().unwrap(), &geo_gw).unwrap();
            }
            let cfg = ofdb_webserver::Cfg {
                accepted_licenses: cfg.accepted_licenses,
                protect_with_captcha: cfg.protect_with_captcha,
            };
            ofdb_webserver::run(
                connections,
                search_engine,
                matches.get_flag(ENABLE_CORS_ARG),
                cfg,
                Box::new(geo_gw),
                Box::new(notify_gw),
                env!("CARGO_PKG_VERSION"),
            )
            .await;
        }
    }
}
