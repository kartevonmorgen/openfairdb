// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>
// Copyright (c) 2018 - 2022 slowtec GmbH <post@slowtec.de>

use std::{env, path::Path};

use clap::{crate_authors, Arg, ArgAction, Command};
use dotenv::dotenv;

use ofdb_core::{
    entities::{Email, MapPoint},
    gateways::{email::EmailGateway, geocode::GeoCodingGateway},
    repositories::EventRepo,
    RepoError,
};

use ofdb_db_sqlite::Connections;
use ofdb_db_tantivy as tantivy;
use ofdb_gateways::{mailgun::Mailgun, notify::Notify, opencage::OpenCage, sendmail::Sendmail};

mod cfg;

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

fn mailgun_gw() -> Option<Mailgun> {
    // TODO: move this to crate::cfg
    let api_key = env::var("MAILGUN_API_KEY");
    let domain = env::var("MAILGUN_DOMAIN");
    let from = env::var("MAIL_GATEWAY_SENDER_ADDRESS");

    if let (Ok(api_key), Ok(mail), Ok(domain)) = (api_key, from, domain) {
        // TODO: move this to crate::cfg
        let api_url = env::var("MAILGUN_API_URL")
            .unwrap_or_else(|_| format!("https://api.eu.mailgun.net/v3/{}/messages", domain));
        // TODO: validate values
        Some(Mailgun {
            from_email: Email::from(mail),
            domain,
            api_key,
            api_url,
        })
    } else {
        None
    }
}

fn sendmail_gw() -> Option<Sendmail> {
    let from = env::var("MAIL_GATEWAY_SENDER_ADDRESS");
    if let Ok(mail) = from {
        // TODO: validate values
        Some(Sendmail::new(Email::from(mail)))
    } else {
        None
    }
}

struct DummyMailGw;

impl EmailGateway for DummyMailGw {
    fn compose_and_send(&self, _recipients: &[Email], _subject: &str, _body: &str) {
        log::debug!("Cannot send emails because no e-mail gateway was configured");
    }
}

fn notification_gateway() -> Notify {
    if let Some(gw) = mailgun_gw() {
        log::info!("Use Mailgun gateway");
        Notify::new(gw)
    } else if let Some(gw) = sendmail_gw() {
        log::warn!("Mailgun gateway was not configured: use sendmail as fallback");
        Notify::new(gw)
    } else {
        log::warn!("No eMail gateway was not configured");
        Notify::new(DummyMailGw)
    }
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

    let geo_gw = OpenCage::new(cfg.opencage_api_key);
    let notify_gw = notification_gateway();

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
