use r2d2_cypher::CypherConnectionManager;
use r2d2::{self, Pool};
use rocket::{self, LoggingLevel};
use rocket_contrib::JSON;
use rocket::response::{content, Response, Responder};
use rocket::http::Status;
use rocket::config::{Environment, Config};
use adapters::json;
use entities::*;
use business::db::Repo;
use adapters::error::Error as AdapterError;
use business::error::{Error, RepoError};
use infrastructure::error::AppError;
use serde_json::ser::to_string;
use business::sort::SortByDistanceTo;
use business::{usecase, filter, geo};
use business::filter::InBBox;
use business::duplicates::{self, DuplicateType};
use std::convert::TryFrom;
use rusted_cypher::GraphClient;
use std::{env,result,io};

static POOL_SIZE: u32 = 5;
static MAX_INVISIBLE_RESULTS: usize = 5;
static DB_URL_KEY: &'static str = "OFDB_DATABASE_URL";

#[derive(Debug, Clone)]
struct Data {
    db: r2d2::Pool<CypherConnectionManager>,
}

lazy_static! {
    pub static ref DB_POOL: r2d2::Pool<CypherConnectionManager> = {
        let config = r2d2::Config::builder().pool_size(POOL_SIZE).build();
        let db_url = env::var(DB_URL_KEY).expect(&format!("{} must be set.", DB_URL_KEY));
        let manager = CypherConnectionManager { url: db_url.into() };
        Pool::new(config, manager).expect("Failed to create pool.")
    };
}

pub struct DB(r2d2::PooledConnection<CypherConnectionManager>);

impl DB {
    pub fn conn(&mut self) -> &mut GraphClient {
        &mut *self.0
    }
}

pub fn db() -> io::Result<DB> {
    match DB_POOL.get() {
        Ok(conn) => Ok(DB(conn)),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}

type Result<T> = result::Result<content::JSON<T>, AppError>;

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
fn get_entries() -> Result<JSON<Vec<json::Entry>>> {
    let entries: Vec<Entry> = db()?.conn().all()?;
    let e = entries
        .into_iter()
        .map(json::Entry::from)
        .collect::<Vec<json::Entry>>();
    Ok(content::JSON(JSON(e)))
}

#[get("/entries/<id>")]
fn get_entry(id: &str) -> Result<String> {
    let ids = extract_ids(id);
    let entries: Vec<Entry> = db()?.conn().all()?;
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
fn get_duplicates() -> Result<JSON<Vec<(String, String, DuplicateType)>>> {
    let entries: Vec<Entry> = db()?.conn().all()?;
    let ids = duplicates::find_duplicates(&entries);
    Ok(content::JSON(JSON(ids)))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(e: JSON<usecase::NewEntry>) -> Result<String> {
    let id = usecase::create_new_entry(db()?.conn(), e.unwrap())?;
    Ok(content::JSON(id))
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(id: &str, e: JSON<usecase::UpdateEntry>) -> Result<String> {
    usecase::update_entry(db()?.conn(), e.unwrap())?;
    Ok(content::JSON(id.to_string()))
}

#[get("/categories")]
fn get_categories() -> Result<JSON<Vec<json::Category>>> {
    let categories: Vec<Category> = db()?.conn().all()?;
    Ok(content::JSON(JSON(categories
        .into_iter()
        .map(json::Category::from)
        .collect::<Vec<json::Category>>())))
}

#[get("/categories/<id>")]
fn get_category(id: &str) -> Result<String> {
    let ids = extract_ids(id);
    let categories: Vec<Category> = db()?.conn().all()?;
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
    Ok(content::JSON(res))
}

#[derive(FromForm)]
struct SearchQuery {
    bbox: String,
    categories: String,
    text: Option<String>,
}

#[get("/search?<search>")]
fn get_search(search: SearchQuery) -> Result<JSON<json::SearchResult>> {
    let entries: Vec<Entry> = db()?.conn().all()?;
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

    Ok(content::JSON(JSON(json::SearchResult {
        visible: visible_results,
        invisible: invisible_results,
    })))
}

#[get("/count/entries")]
fn get_count_entries() -> Result<String> {
    let entries: Vec<Entry> = db()?.conn().all()?;
    Ok(content::JSON(entries.len().to_string()))
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

    let cfg = Config::default_for(Environment::Production, "/custom")
        .unwrap()
        .log_level(LoggingLevel::Normal)
        .address("127.0.0.1".into())
        .port(port as usize);

    rocket::custom(&cfg)
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
            AppError::Parse(_) |
            AppError::Adapter(_) => Status::BadRequest,

            _ => Status::InternalServerError,
        })
    }
}
