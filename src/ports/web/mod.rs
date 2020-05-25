use crate::{
    core::{
        db::{EventIndexer, PlaceIndexer},
        prelude::*,
        usecases,
        util::sort::Rated,
    },
    infrastructure::error::AppError,
};
use rocket::{config::Config, Rocket, Route};
use rocket_contrib::json::Json;
use std::result;

pub mod api;
#[cfg(feature = "frontend")]
mod frontend;
mod guards;
#[cfg(test)]
mod mockdb;
mod sqlite;
mod tantivy;

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
    cfg: Option<Config>,
) -> Rocket {
    info!("Indexing all places...");
    index_all_places(&*connections.exclusive().unwrap(), &mut search_engine).unwrap();

    info!("Indexing all events...");
    index_all_events_chronologically(&*connections.exclusive().unwrap(), &mut search_engine)
        .unwrap();

    info!("Deleting expired user e-mail tokens...");
    usecases::delete_expired_user_tokens(&*connections.exclusive().unwrap()).unwrap();

    info!("Initialization finished");
    let r = match cfg {
        Some(cfg) => rocket::custom(cfg),
        None => rocket::ignite(),
    };
    let mut instance = r.manage(connections).manage(search_engine);

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
) {
    if enable_cors {
        let cors = rocket_cors::CorsOptions {
            ..Default::default()
        }.to_cors().unwrap();
        rocket_instance(connections, search_engine, mounts(), None)
            .attach(cors).launch();
    } else {
        rocket_instance(connections, search_engine, mounts(), None).launch();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{prelude::*, usecases};
    use crate::infrastructure::db::{sqlite, tantivy};
    use rocket::{
        config::{Config, Environment},
        local::Client,
        logger::LoggingLevel,
        Route,
    };

    pub mod prelude {
        pub use crate::core::db::*;
        pub use rocket::{
            http::{ContentType, Cookie, Status},
            local::Client,
            response::Response,
        };
    }

    embed_migrations!();

    pub fn setup(
        mounts: Vec<(&'static str, Vec<Route>)>,
    ) -> (
        rocket::local::Client,
        sqlite::Connections,
        tantivy::SearchEngine,
    ) {
        let cfg = Config::build(Environment::Development)
            .log_level(LoggingLevel::Debug)
            .finalize()
            .unwrap();
        let connections = sqlite::Connections::init(":memory:", 1).unwrap();
        embedded_migrations::run(&*connections.exclusive().unwrap()).unwrap();
        let search_engine = tantivy::SearchEngine::init_in_ram().unwrap();
        let rocket = super::rocket_instance(
            connections.clone(),
            search_engine.clone(),
            mounts,
            Some(cfg),
        );
        let client = Client::new(rocket).unwrap();
        (client, connections, search_engine)
    }

    pub fn register_user(pool: &sqlite::Connections, email: &str, pw: &str, confirmed: bool) {
        let db = pool.exclusive().unwrap();
        usecases::create_new_user(
            &*db,
            usecases::NewUser {
                email: email.to_string(),
                password: pw.to_string(),
            },
        )
        .unwrap();
        let email_nonce = EmailNonce {
            email: email.to_string(),
            nonce: Nonce::new(),
        };
        let token = email_nonce.encode_to_string();
        if confirmed {
            usecases::confirm_email_address(&*db, &token).unwrap();
        }
    }
}
