use clap::{App, Arg, SubCommand};
use super::web;
use super::osm;
use dotenv::dotenv;
use std::{env, process};

const DEFAULT_DB_URL: &str = "openfair.db";

pub fn run() {
    dotenv().ok();
    let matches = App::new("openFairDB")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Markus Kohlhase <mail@markus-kohlhase.de>")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value("6767")
                .help("Set the port to listen"),
        )
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
            let port = match matches.value_of("port") {
                Some(port) => port.parse::<u16>().unwrap(),
                None => {
                    println!("{}", matches.usage());
                    process::exit(1)
                }
            };

            web::run(&db_url, port, matches.is_present("enable-cors"));
        }
    }
}
