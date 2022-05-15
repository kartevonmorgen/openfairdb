use std::result;

use ofdb_core::rating::Rated;
use popular_tags_cache::PopularTagsCache;
use rocket::{config::Config as RocketCfg, Rocket, Route};
use rocket_contrib::json::Json;

use crate::{
    core::{
        db::{EventIndexer, PlaceIndexer},
        prelude::*,
        usecases,
    },
    infrastructure::{cfg::Cfg, error::AppError},
};

pub mod api;
#[cfg(feature = "frontend")]
mod frontend;
mod guards;
pub mod jwt;
#[cfg(test)]
mod mockdb;
pub mod notify;
mod popular_tags_cache;
mod sqlite;
mod tantivy;
#[cfg(test)]
pub mod tests;

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

fn index_all_events_chronologically<D: EventGateway>(
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
) -> Rocket {
    info!("Indexing all places...");
    index_all_places(&*connections.exclusive().unwrap(), &mut search_engine).unwrap();

    info!("Indexing all events...");
    index_all_events_chronologically(&*connections.exclusive().unwrap(), &mut search_engine)
        .unwrap();

    info!("Deleting expired user e-mail tokens...");
    usecases::delete_expired_user_tokens(&*connections.exclusive().unwrap()).unwrap();

    info!("Caching most popular tags...");
    let tags_cache = PopularTagsCache::new_from_db(&*connections.shared().unwrap()).unwrap();

    let captcha_cache = api::captcha::CaptchaCache::new();
    let jwt_state = jwt::JwtState::new();

    info!("Initialization finished");

    let r = match rocket_cfg {
        Some(cfg) => rocket::custom(cfg),
        None => rocket::ignite(),
    };

    let mut instance = r
        .manage(connections)
        .manage(search_engine)
        .manage(captcha_cache)
        .manage(tags_cache)
        .manage(jwt_state)
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

pub fn run(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    enable_cors: bool,
    cfg: Cfg,
) {
    if enable_cors {
        let cors = rocket_cors::CorsOptions {
            ..Default::default()
        }
        .to_cors()
        .unwrap();
        rocket_instance(connections, search_engine, mounts(), None, cfg)
            .attach(cors)
            .launch();
    } else {
        rocket_instance(connections, search_engine, mounts(), None, cfg).launch();
    }
}
