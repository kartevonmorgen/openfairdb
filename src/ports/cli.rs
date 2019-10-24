use super::web;

use crate::core::prelude::*;
use crate::infrastructure::{
    db::{sqlite, tantivy},
    osm,
};

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use std::{env, path::Path, process};

const DEFAULT_DB_URL: &str = "openfair.db";
const DB_CONNECTION_POOL_SIZE: u32 = 10;

embed_migrations!();

fn update_event_locations<D: Db>(db: &mut D) -> Result<()> {
    let events = db.all_events()?;
    for mut e in events {
        if let Some(ref mut loc) = e.location {
            if let Some(ref addr) = loc.address {
                if let Some((lat, lng)) = web::api::geocoding::resolve_address_lat_lng(addr) {
                    if let Some(pos) = MapPoint::try_from_lat_lng_deg(lat, lng) {
                        if pos.is_valid() {
                            if let Err(err) = db.update_event(&e) {
                                warn!("Failed to update location of event {}: {}", e.uid, err);
                            } else {
                                info!("Updated location of event {}", e.uid);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn run() {
    dotenv().ok();
    let matches = App::new("openFairDB")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Markus Kohlhase <mail@markus-kohlhase.de>")
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
        .subcommand(
            SubCommand::with_name("osm")
                .about("OpenStreetMap functionalities")
                .subcommand(
                    SubCommand::with_name("import")
                        .about("import entries from OSM (JSON file)")
                        .arg(
                            Arg::with_name("osm-file")
                                .value_name("OSM_FILE")
                                .help("JSON file with osm nodes"),
                        ),
                ),
        )
        .get_matches();

    let db_url = matches
        .value_of("db-url")
        .map(ToString::to_string)
        .unwrap_or_else(|| env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DB_URL.to_string()));
    info!(
        "Connecting to SQLite database '{}' (pool size = {})",
        db_url, DB_CONNECTION_POOL_SIZE
    );
    let connections = sqlite::Connections::init(&db_url, DB_CONNECTION_POOL_SIZE).unwrap();

    info!("Running embedded database migrations");
    embedded_migrations::run(&*connections.exclusive().unwrap()).unwrap();

    let idx_dir = matches
        .value_of("idx-dir")
        .map(ToString::to_string)
        .or_else(|| env::var("INDEX_DIR").map(Option::Some).unwrap_or(None));
    let idx_path = idx_dir.as_ref().map(|dir| Path::new(dir));
    info!("Initializing Tantivy full-text search engine");
    let search_engine = tantivy::SearchEngine::init_with_path(idx_path).unwrap();

    match matches.subcommand() {
        ("osm", Some(osm_matches)) => match osm_matches.subcommand() {
            ("import", Some(import_matches)) => {
                let osm_file = match import_matches.value_of("osm-file") {
                    Some(osm_file) => osm_file,
                    None => {
                        println!("{}", matches.usage());
                        process::exit(1)
                    }
                };
                if let Err(err) = osm::import_from_osm_file(&db_url, osm_file) {
                    println!("Could not import from '{}': {}", osm_file, err);
                    process::exit(1)
                }
            }
            _ => println!("{}", osm_matches.usage()),
        },
        _ => {
            if matches.is_present("fix-event-address-location") {
                info!("Updating all event locations...");
                update_event_locations(&mut *connections.exclusive().unwrap()).unwrap();
            }
            web::run(
                connections,
                search_engine,
                matches.is_present("enable-cors"),
            );
        }
    }
}
