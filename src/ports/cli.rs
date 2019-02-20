use super::web::{self, sqlite::create_connection_pool, tantivy::create_search_engine};
use crate::core::prelude::*;
use crate::infrastructure::osm;
use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use std::{env, process};

const DEFAULT_DB_URL: &str = "openfair.db";

fn update_event_locations<D: Db>(db: &mut D) -> Result<()> {
    let events = db.all_events()?;
    for mut e in events {
        if let Some(ref mut loc) = e.location {
            if let Some(ref addr) = loc.address {
                if let Some((lat, lng)) = web::api::geocoding::resolve_address_lat_lng(addr) {
                    loc.lat = lat;
                    loc.lng = lng;
                    if let Err(err) = db.update_event(&e) {
                        warn!("Failed to update location of event {}: {}", e.id, err);
                    } else {
                        info!("Updated location of event {}", e.id);
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

    let db_url = match matches.value_of("db-url") {
        Some(db_url) => db_url.into(),
        None => match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => DEFAULT_DB_URL.to_string(),
        },
    };
    let pool = create_connection_pool(&db_url).unwrap();

    let search_engine = create_search_engine().unwrap();

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
                update_event_locations(&mut *pool.get().unwrap()).unwrap();
            }
            web::run(pool, search_engine, matches.is_present("enable-cors"));
        }
    }
}
