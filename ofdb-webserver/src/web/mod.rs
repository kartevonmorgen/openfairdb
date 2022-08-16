use std::{collections::HashSet, result};

use crate::core::{
    db::{EventIndexer, PlaceIndexer},
    prelude::*,
    usecases,
};

use ofdb_application::error::AppError;
use ofdb_core::{
    gateways::{geocode::GeoCodingGateway, notify::NotificationGateway},
    rating::Rated,
};

use rocket::{config::Config as RocketCfg, serde::json::Json, Rocket, Route};

pub mod api;
#[cfg(feature = "frontend")]
mod frontend;
mod guards;
pub mod jwt;
mod popular_tags_cache;
mod sqlite;
pub mod tantivy;

#[cfg(test)]
mod mockdb;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub accepted_licenses: HashSet<String>,
    pub protect_with_captcha: bool,
}

use popular_tags_cache::PopularTagsCache;

type Result<T> = result::Result<Json<T>, AppError>;

fn index_all_places<D: PlaceRepo + RatingRepository>(
    db: &D,
    indexer: &mut dyn PlaceIndexer,
) -> Result<()> {
    // TODO: Split into chunks with fixed size instead of
    // loading all places at once!
    let places = db.all_places()?;
    for (place, status) in places {
        let ratings = db.load_ratings_of_place(place.id.as_ref())?;
        if let Err(err) =
            indexer.add_or_update_place(&place, status, &place.avg_ratings(&ratings[..]))
        {
            error!("Failed to index place {:?}: {}", place, err);
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!("Failed to build place index: {}", err);
    }
    Ok(Json(()))
}

fn index_all_events_chronologically<D: EventRepo>(
    db: &D,
    indexer: &mut dyn EventIndexer,
) -> Result<()> {
    // TODO: Split into chunks with fixed size instead of
    // loading all events at once!
    let events = db.all_events_chronologically()?;
    for event in events {
        if let Err(err) = indexer.add_or_update_event(&event) {
            error!("Failed to index event {:?}: {}", event, err);
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!("Failed to build event index: {}", err);
    }
    Ok(Json(()))
}

pub(crate) fn rocket_instance(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    mounts: Vec<(&str, Vec<Route>)>,
    rocket_cfg: Option<RocketCfg>,
    cfg: Cfg,
    geo_gw: Box<dyn GeoCodingGateway + Send + Sync>,
    notify_gw: Box<dyn NotificationGateway + Send + Sync>,
) -> Rocket<rocket::Build> {
    info!("Indexing all places...");
    index_all_places(&connections.exclusive().unwrap(), &mut *search_engine).unwrap();

    info!("Indexing all events...");
    index_all_events_chronologically(&connections.exclusive().unwrap(), &mut *search_engine)
        .unwrap();

    info!("Deleting expired user e-mail tokens...");
    usecases::delete_expired_user_tokens(&connections.exclusive().unwrap()).unwrap();

    info!("Caching most popular tags...");
    let tags_cache = PopularTagsCache::new_from_db(&connections.shared().unwrap()).unwrap();

    let captcha_cache = api::captcha::CaptchaCache::new();
    let jwt_state = jwt::JwtState::new();

    info!("Initialization finished");

    let r = match rocket_cfg {
        Some(cfg) => rocket::custom(cfg),
        None => rocket::build(),
    };

    let geo_gw = guards::GeoCoding(geo_gw);
    let notify_gw = guards::Notify(notify_gw);

    let mut instance = r
        .manage(connections)
        .manage(search_engine)
        .manage(captcha_cache)
        .manage(tags_cache)
        .manage(jwt_state)
        .manage(geo_gw)
        .manage(notify_gw)
        .manage(cfg);

    for (m, r) in mounts {
        instance = instance.mount(m, r);
    }
    instance
}

#[cfg(not(feature = "frontend"))]
fn mounts() -> Vec<(&'static str, Vec<Route>)> {
    vec![("/api", api::routes())]
}

#[cfg(feature = "frontend")]
fn mounts() -> Vec<(&'static str, Vec<Route>)> {
    vec![("/api", api::routes()), ("/", frontend::routes())]
}

pub async fn run(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    enable_cors: bool,
    cfg: Cfg,
    geo_gw: Box<dyn GeoCodingGateway + Send + Sync>,
    notify_gw: Box<dyn NotificationGateway + Send + Sync>,
) {
    let server_task = if enable_cors {
        let cors = rocket_cors::CorsOptions {
            ..Default::default()
        }
        .to_cors()
        .unwrap();
        rocket_instance(
            connections,
            search_engine,
            mounts(),
            None,
            cfg,
            geo_gw,
            notify_gw,
        )
        .attach(cors)
        .launch()
    } else {
        rocket_instance(
            connections,
            search_engine,
            mounts(),
            None,
            cfg,
            geo_gw,
            notify_gw,
        )
        .launch()
    };
    if let Err(err) = server_task.await {
        log::error!("Unable to run web server: {err}");
    }
}
