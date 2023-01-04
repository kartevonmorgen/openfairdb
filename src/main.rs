// Copyright (c) 2018 - 2023 slowtec GmbH <post@slowtec.de>
// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>

use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};
use dotenv::dotenv;

use ofdb_core::{
    entities::MapPoint, gateways::geocode::GeoCodingGateway, repositories::EventRepo, RepoError,
};

use ofdb_db_sqlite::Connections;
use ofdb_db_tantivy as tantivy;

mod config;
mod gateways;
mod recurring_reminder;

fn update_event_locations<R, G>(repo: &R, geo: &G) -> Result<(), RepoError>
where
    R: EventRepo,
    G: GeoCodingGateway + ?Sized,
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

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Configuration file
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Update the location of ALL events by resolving their address
    FixEventAddressLocation,
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    env_logger::init();
    dotenv().ok();
    let args = Args::parse();

    let cfg = config::Config::try_load_from_file_or_default(args.config_file)?;
    let config::Db {
        conn_sqlite,
        conn_pool_size,
        index_dir,
    } = cfg.db;

    log::info!("Connecting to SQLite database '{conn_sqlite}' (pool size = {conn_pool_size})");
    let connections = Connections::init(&conn_sqlite, u32::from(conn_pool_size)).unwrap();

    ofdb_db_sqlite::run_embedded_database_migrations(connections.exclusive().unwrap());

    log::info!("Initializing Tantivy full-text search engine");
    let search_engine = tantivy::SearchEngine::init_with_path(index_dir).unwrap();

    let geo_gw = gateways::geocoding_gateway(cfg.geocoding.gateway);
    let notify_gw = gateways::notification_gateway(cfg.email.gateway.clone());

    let db_connections = connections.clone();

    tokio::spawn(async move {
        recurring_reminder::run(&db_connections, cfg.email.gateway, cfg.reminders).await;
    });

    match args.command {
        Some(cmd) => match cmd {
            Command::FixEventAddressLocation => {
                log::info!("Updating all event locations...");
                update_event_locations(&connections.exclusive().unwrap(), &*geo_gw).unwrap();
            }
        },
        None => {
            let web_server_cfg = ofdb_webserver::Cfg {
                accepted_licenses: cfg.entries.accepted_licenses,
                protect_with_captcha: cfg.webserver.protect_with_captcha,
            };
            ofdb_webserver::run(
                connections,
                search_engine,
                cfg.webserver.enable_cors, // TODO: move this to ofdb_webserver::Cfg
                web_server_cfg,
                geo_gw,
                Box::new(notify_gw),
                env!("CARGO_PKG_VERSION"),
            )
            .await;
        }
    }
    Ok(())
}
