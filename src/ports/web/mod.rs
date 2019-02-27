use crate::core::{db::EntryIndexer, prelude::*, util::sort::Rated};
use crate::infrastructure::error::AppError;
use rocket::{config::Config, Rocket};
use rocket_contrib::json::Json;
use std::result;

pub mod api;
#[cfg(test)]
mod mockdb;
mod sqlite;
mod tantivy;
#[cfg(test)]
pub use self::api::tests;
#[cfg(feature = "frontend")]
mod frontend;
mod guards;
mod util;

type Result<T> = result::Result<Json<T>, AppError>;

fn index_all_entries<D: EntryGateway + EntryRatingRepository>(
    db: &D,
    entry_indexer: &mut dyn EntryIndexer,
) -> Result<()> {
    // TODO: Split into batches with limited size instead of
    // loading all entries at once!
    let entries = db.all_entries()?;
    for entry in entries {
        let ratings = db.all_ratings_for_entry_by_id(&entry.id)?;
        if let Err(err) =
            entry_indexer.add_or_update_entry(&entry, &entry.avg_ratings(&ratings[..]))
        {
            error!("Failed to index entry {:?}: {}", entry, err);
        }
    }
    if let Err(err) = entry_indexer.flush() {
        error!("Failed to build entry index: {}", err);
    }
    Ok(Json(()))
}

fn rocket_instance(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    cfg: Option<Config>,
) -> Rocket {
    info!("Indexing all entries...");
    index_all_entries(&*connections.exclusive().unwrap(), &mut search_engine).unwrap();

    info!("Initialization finished");
    let r = match cfg {
        Some(cfg) => rocket::custom(cfg),
        None => rocket::ignite(),
    };
    r.manage(connections)
        .manage(search_engine)
        .mount("/", api::routes())
        .mount("/frontend", frontend::routes()) // TODO don't mount if feature "frontend" is disabled
}

pub fn run(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    enable_cors: bool,
) {
    if enable_cors {
        panic!(
            "enable-cors is currently not available until\
             \nhttps://github.com/SergioBenitez/Rocket/pull/141\nis merged :("
        );
    }

    rocket_instance(connections, search_engine, None).launch();
}
