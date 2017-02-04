use clap::{Arg, App};
use super::web;

pub fn run() {
    let matches = App::new("openFairDB")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Markus Kohlhase <mail@markus-kohlhase.de>")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .default_value("6767")
            .help("Set the port to listen"))
        .arg(Arg::with_name("db-url")
            .long("db-url")
            .value_name("URL")
            .default_value("http://neo4j:neo4j@127.0.0.1:7474/db/data")
            .help("URL to the Neo4j database"))
        .arg(Arg::with_name("enable-cors")
            .long("enable-cors")
            .help("Allow requests from any origin"))
        .get_matches();

    match matches.value_of("db-url") {
        Some(db_url) => {
            match matches.value_of("port") {
                Some(port) => {
                    let port = port.parse::<u16>().unwrap();
                    web::run(db_url, port, matches.is_present("enable-cors"));
                }
                None => println!("{}", matches.usage()),
            }
        }
        None => println!("{}", matches.usage()),
    }
}
