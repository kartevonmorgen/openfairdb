use core::{prelude::*, util::sort::Rated};
use diesel::r2d2::{self, Pool};
use infrastructure::error::AppError;
use rocket::{self,
             config::{Config, Environment},
             Rocket};
use rocket_contrib::Json;
use std::{collections::HashMap, result, sync::Mutex};

#[cfg(feature = "email")]
use infrastructure::mail;

lazy_static! {
    static ref ENTRY_RATINGS: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}

mod api;
#[cfg(test)]
mod mockdb;
pub mod sqlite;
#[cfg(test)]
mod tests;
mod util;

use self::sqlite::create_connection_pool;

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

fn rocket_instance<T: r2d2::ManageConnection>(cfg: Config, pool: Pool<T>) -> Rocket
where
    <T as r2d2::ManageConnection>::Connection: Db,
{
    info!("Calculating the average rating of all entries...");
    calculate_all_ratings(&*pool.get().unwrap()).unwrap();
    info!("done.");
    rocket::custom(cfg, true)
        .manage(pool)
        .mount("/", api::routes())
}

pub fn run(db_url: &str, port: u16, enable_cors: bool) {
    if enable_cors {
        panic!(
            "enable-cors is currently not available until\
             \nhttps://github.com/SergioBenitez/Rocket/pull/141\nis merged :("
        );
    }

    let cfg = Config::build(Environment::Production)
        .address("127.0.0.1")
        .port(port)
        .finalize()
        .unwrap();

    let pool = create_connection_pool(db_url).unwrap();

    rocket_instance(cfg, pool).launch();
}
