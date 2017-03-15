use rocket::{self, LoggingLevel};
use rocket_contrib::JSON;
use rocket::response::{Response, content, Responder};
use rocket::http::Status;
use rocket::config::{Environment, Config};
use adapters::json;
use entities::*;
use business::db::Db;
use business::error::{Error, RepoError};
use infrastructure::error::AppError;
use serde_json::ser::to_string;
use business::sort::SortByDistanceTo;
use business::{usecase, filter, geo};
use business::filter::InBBox;
use business::duplicates::{self, DuplicateType};
use std::{env,result,io};

static MAX_INVISIBLE_RESULTS: usize = 5;
static DB_URL_KEY: &'static str = "OFDB_DATABASE_URL";

#[cfg(not(test))]
mod neo4j;
#[cfg(test)]
mod mockdb;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
fn db() -> io::Result<neo4j::DB> { neo4j::db() }
#[cfg(test)]
fn db() -> io::Result<mockdb::DB> { mockdb::db() }

type Result<T> = result::Result<JSON<T>, AppError>;

fn extract_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.to_owned())
        .filter(|id| id != "")
        .collect::<Vec<String>>()
}

#[test]
fn extract_ids_test() {
    assert_eq!(extract_ids("abc"), vec!["abc"]);
    assert_eq!(extract_ids("a,b,c"), vec!["a", "b", "c"]);
    assert_eq!(extract_ids("").len(), 0);
    assert_eq!(extract_ids("abc,,d"), vec!["abc", "d"]);
}

#[get("/entries")]
fn get_entries() -> Result<Vec<json::Entry>> {
    let e = db()?
        .conn()
        .all_entries()?
        .into_iter()
        .map(json::Entry::from)
        .collect::<Vec<json::Entry>>();
    Ok(JSON(e))
}

#[get("/entries/<id>")]
fn get_entry(id: &str) -> result::Result<content::JSON<String>,AppError> {
    let ids = extract_ids(id);
    let entries = db()?.conn().all_entries()?;
    let e = match ids.len() {
        0 => {
            to_string(&entries.into_iter()
                .map(json::Entry::from)
                .collect::<Vec<json::Entry>>())
        }
        1 => {
            let e = entries.iter()
                .find(|x| x.id == ids[0].clone())
                .ok_or(RepoError::NotFound)?;
            to_string(&json::Entry::from(e.clone()))
        }
        _ => {
            to_string(&entries.into_iter()
                .filter(|e| ids.iter().any(|id| e.id == id.clone()))
                .map(json::Entry::from)
                .collect::<Vec<json::Entry>>())
        }
    }?;
    Ok(content::JSON(e))
}

#[get("/duplicates")]
fn get_duplicates() -> Result<Vec<(String, String, DuplicateType)>> {
    let entries = db()?.conn().all_entries()?;
    let ids = duplicates::find_duplicates(&entries);
    Ok(JSON(ids))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(e: JSON<usecase::NewEntry>) -> result::Result<String,AppError> {
    let id = usecase::create_new_entry(db()?.conn(), e.into_inner())?;
    Ok(id)
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(id: &str, e: JSON<usecase::UpdateEntry>) -> Result<String> {
    usecase::update_entry(db()?.conn(), e.into_inner())?;
    Ok(JSON(id.to_string()))
}

#[get("/categories")]
fn get_categories() -> Result<Vec<json::Category>> {
    let categories = db()?.conn().all_categories()?;
    Ok(JSON(categories
        .into_iter()
        .map(json::Category::from)
        .collect::<Vec<json::Category>>()))
}

#[get("/categories/<id>")]
fn get_category(id: &str) -> Result<String> {
    let ids = extract_ids(id);
    let categories = db()?.conn().all_categories()?;
    let res = match ids.len() {
        0 => {
            to_string(&categories.into_iter()
                .map(json::Category::from)
                .collect::<Vec<json::Category>>())
        }
        1 => {
            let id = ids[0].clone();
            let e = categories.into_iter()
                .find(|x| x.id == id)
                .ok_or(RepoError::NotFound)?;
            to_string(&json::Category::from(e))
        }
        _ => {
            to_string(&categories.into_iter()
                .filter(|e| ids.iter().any(|id| e.id == id.clone()))
                .map(json::Category::from)
                .collect::<Vec<json::Category>>())
        }
    }?;
    Ok(JSON(res))
}

#[derive(FromForm)]
struct SearchQuery {
    bbox: String,
    categories: String,
    text: Option<String>,
}

#[get("/search?<search>")]
fn get_search(search: SearchQuery) -> Result<json::SearchResult> {
    let entries: Vec<Entry> = db()?.conn().all_entries()?;
    let cat_ids = extract_ids(&search.categories);
    let bbox = geo::extract_bbox(&search.bbox).map_err(Error::Parameter)
        .map_err(AppError::Business)?;
    let bbox_center = geo::center(&bbox[0], &bbox[1]);

    let entries: Vec<_> = entries.into_iter()
        .filter(|x| filter::entries_by_category_ids(&cat_ids)(&x))
        .collect();

    let mut entries = match search.text {
        Some(txt) => {
            entries.into_iter().filter(|x| filter::entries_by_search_text(&txt)(&x)).collect()
        }
        None => entries,
    };

    entries.sort_by_distance_to(&bbox_center);

    let visible_results: Vec<_> =
        entries.iter().filter(|x| x.in_bbox(&bbox)).map(|x| x.id.clone()).collect();

    let invisible_results = entries.iter()
        .filter(|e| !visible_results.iter().any(|v| *v == e.id))
        .take(MAX_INVISIBLE_RESULTS)
        .map(|x| x.id.clone())
        .collect::<Vec<_>>();

    Ok(JSON(json::SearchResult {
        visible: visible_results,
        invisible: invisible_results,
    }))
}

#[get("/count/entries")]
fn get_count_entries() -> Result<String> {
    let entries = db()?.conn().all_entries()?;
    Ok(JSON(entries.len().to_string()))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn run(db_url: &str, port: u16, enable_cors: bool) {

    env::set_var(DB_URL_KEY, db_url);

    if enable_cors {
        panic!("This feature is currently not available until\
        \nhttps://github.com/SergioBenitez/Rocket/pull/141\nis merged :(");
    }

    let cfg = Config::build(Environment::Production)
        .log_level(LoggingLevel::Normal)
        .address("127.0.0.1")
        .port(port)
        .finalize()
        .unwrap();

    rocket::custom(cfg,true)
        .mount("/",
               routes![get_entries,
                       get_entry,
                       post_entry,
                       put_entry,
                       get_categories,
                       get_category,
                       get_search,
                       get_duplicates,
                       get_count_entries,
                       get_version])
        .launch();
}

impl<'r> Responder<'r> for AppError {
    fn respond(self) -> result::Result<Response<'r>, Status> {
        Err(match self {
            AppError::Business(ref err) => {
                match *err {
                    Error::Parameter(_) => Status::BadRequest,
                    Error::Repo(ref err) => {
                        match *err {
                            RepoError::NotFound => Status::NotFound,
                            _ => Status::InternalServerError,
                        }
                    }
                }
            }
            AppError::Adapter(_) => Status::BadRequest,

            _ => Status::InternalServerError,
        })
    }
}
