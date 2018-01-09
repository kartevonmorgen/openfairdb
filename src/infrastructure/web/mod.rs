use rocket::{self, Rocket};
use rocket_contrib::Json;
use rocket::config::{Environment, Config};
use business::db::Db;
use infrastructure::error::AppError;
use business::sort::Rated;
use std::result;
use diesel::r2d2::{self, Pool};
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "email")]
use super::mail;

lazy_static! {
    static ref ENTRY_RATINGS: Mutex<HashMap<String, f64>> = Mutex::new(HashMap::new());
}

mod routes;
mod util;
pub mod sqlite;
#[cfg(test)]
mod mockdb;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
use self::sqlite::create_connection_pool;

#[cfg(test)]
use self::mockdb::create_connection_pool;

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
    rocket::custom(cfg, true).manage(pool).mount(
        "/",
        routes![
            routes::login,
            routes::logout,
            // send_confirmation_email,
            routes::delete_user,
            routes::confirm_email_address,
            routes::subscribe_to_bbox,
            routes::get_bbox_subscriptions,
            routes::unsubscribe_all_bboxes,
            routes::get_entry,
            routes::post_entry,
            routes::post_user,
            routes::post_rating,
            routes::put_entry,
            routes::get_user,
            routes::get_categories,
            routes::get_tags,
            routes::get_ratings,
            routes::get_category,
            routes::get_search,
            routes::get_duplicates,
            routes::get_count_entries,
            routes::get_count_tags,
            routes::get_version,
        ],
    )
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
