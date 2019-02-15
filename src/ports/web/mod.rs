use crate::core::{prelude::*, util::sort::Rated};
use crate::infrastructure::error::AppError;
use diesel::r2d2::{ManageConnection, Pool};
use rocket::{config::Config, Rocket};
use rocket_contrib::json::Json;
use std::{collections::HashMap, result, sync::Mutex};

#[cfg(feature = "email")]
use crate::infrastructure::mail;

lazy_static! {
    static ref ENTRY_RATINGS: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}

pub mod api;
#[cfg(test)]
mod mockdb;
pub mod sqlite;
#[cfg(test)]
pub use self::api::tests;
mod guards;
mod util;

type Result<T> = result::Result<Json<T>, AppError>;

fn calculate_all_ratings<D: Db>(db: &D) -> Result<()> {
    let entries = db.all_entries()?;
    let ratings = db.all_ratings()?;
    let mut avg_ratings = match ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    for e in entries {
        avg_ratings.insert(e.id.clone(), e.avg_rating(&ratings));
    }
    Ok(Json(()))
}

fn calculate_rating_for_entry<D: Db>(db: &D, e_id: &str) -> Result<()> {
    let ratings = db.all_ratings()?;
    let e = db.get_entry(e_id)?;
    let mut avg_ratings = match ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    avg_ratings.insert(e.id.clone(), e.avg_rating(&ratings));
    Ok(Json(()))
}

fn rocket_instance<T: ManageConnection>(pool: Pool<T>, cfg: Option<Config>) -> Rocket
where
    <T as ManageConnection>::Connection: Db,
{
    info!("Calculating the average rating of all entries...");
    calculate_all_ratings(&*pool.get().unwrap()).unwrap();

    info!("Initialization finished");
    let r = match cfg {
        Some(cfg) => rocket::custom(cfg),
        None => rocket::ignite(),
    };
    r.manage(pool).mount("/", api::routes())
}

pub fn run<T: ManageConnection>(pool: Pool<T>, enable_cors: bool)
where
    <T as ManageConnection>::Connection: Db,
{
    if enable_cors {
        panic!(
            "enable-cors is currently not available until\
             \nhttps://github.com/SergioBenitez/Rocket/pull/141\nis merged :("
        );
    }

    rocket_instance(pool, None).launch();
}
