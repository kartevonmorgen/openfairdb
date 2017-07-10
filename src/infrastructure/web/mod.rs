use rocket::{self, Rocket, State };
use rocket::logger::LoggingLevel;
use rocket_contrib::JSON;
use rocket::response::{Response, Responder};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::http::{Status, Cookie, Cookies};
use rocket::config::{Environment, Config};
use adapters::json;
use entities::*;
use business::db::Db;
use business::error::{Error, RepoError, ParameterError};
use infrastructure::error::AppError;
use serde_json::ser::to_string;
use business::sort::SortByAverageRating;
use business::{usecase, filter, geo};
use business::filter::InBBox;
use business::duplicates::{self, DuplicateType};
use std::{result,thread,env};
use r2d2::{self, Pool};
use regex::Regex;
use super::mail;
use super::cfg;

static MAX_INVISIBLE_RESULTS : usize = 5;
static COOKIE_USER_KEY       : &str  = "user_id";

lazy_static! {
    static ref CONFIG: cfg::Config = {
        match env::current_dir() {
            Ok(cwd) => {
                let path = cwd.as_path().join("config.toml");
                match cfg::Config::load(path) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        warn!("could not read configuration file 'config.toml': {}", e);
                        cfg::Config::default()
                    }
                }
            }
            Err(e) => {
                warn!("could not determine current working directory: {}", e);
                cfg::Config::default()
            }
        }
    };
}

mod neo4j;
#[cfg(test)]
mod mockdb;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
type DbPool = neo4j::ConnectionPool;
#[cfg(test)]
type DbPool = mockdb::ConnectionPool;

type Result<T> = result::Result<JSON<T>, AppError>;

fn extract_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.to_owned())
        .filter(|id| id != "")
        .collect::<Vec<String>>()
}

#[derive(Debug)]
struct User(String);

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let user = request.cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User(id));
        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

fn notify(subject: &str, body: &str) {
    match mail::create(&CONFIG.notification.send_to, subject, body) {
        Ok(mail) => {
            thread::spawn(move ||{
                if let Err(err) = mail::send(&mail) {
                    warn!("Could not send mail: {}", err);
                }
            });
        }
        Err(e) => {
            warn!("could not create notifiction mail: {}", e);
        }
    }
}

#[get("/entries/<ids>")]
fn get_entry(db: State<DbPool>, ids: String) -> Result<Vec<json::Entry>> {
    let ids = extract_ids(&ids);
    let entries = usecase::get_entries(&*db.get()?, &ids)?;
    let tags = usecase::get_tags_by_entry_ids(&*db.get()?, &ids)?;
    let ratings = usecase::get_ratings_by_entry_ids(&*db.get()?, &ids)?;
    Ok(JSON(entries
        .into_iter()
        .map(|e|{
            let t = tags.get(&e.id).cloned().unwrap_or_else(|| vec![]);
            let r = ratings.get(&e.id).cloned().unwrap_or_else(|| vec![]);
            json::Entry::from_entry_with_tags_and_ratings(e,t,r)
        })
        .collect::<Vec<json::Entry>>()))
}

#[get("/duplicates")]
fn get_duplicates(db: State<DbPool>) -> Result<Vec<(String, String, DuplicateType)>> {
    let entries = db.get()?.all_entries()?;
    let ids = duplicates::find_duplicates(&entries);
    Ok(JSON(ids))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(db: State<DbPool>, e: JSON<usecase::NewEntry>) -> result::Result<String, AppError> {
    let e = e.into_inner();
    let id = usecase::create_new_entry(&mut *db.get()?, e.clone())?;
    notify(&format!("Neuer Eintrag: {}", e.title),&format!("{:?}",e));
    Ok(id)
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(db: State<DbPool>, id: String, e: JSON<usecase::UpdateEntry>) -> Result<String> {
    let e = e.into_inner();
    usecase::update_entry(&mut *db.get()?, e.clone())?;
    notify(&format!("Veränderter Eintrag: {}", e.title),&format!("{:?}",e));
    Ok(JSON(id))
}

#[get("/tags")]
fn get_tags(db: State<DbPool>) -> Result<Vec<String>> {
    let tags = usecase::get_tag_ids(&*db.get()?)?;
    Ok(JSON(tags))
}

#[get("/categories")]
fn get_categories(db: State<DbPool>) -> Result<Vec<Category>> {
    let categories = db.get()?.all_categories()?;
    Ok(JSON(categories))
}

#[get("/categories/<id>")]
fn get_category(db: State<DbPool>, id: String) -> Result<String> {
    let ids = extract_ids(&id);
    let categories = db.get()?.all_categories()?;
    let res = match ids.len() {
        0 => to_string(&categories),

        1 => {
            let id = ids[0].clone();
            let e = categories
                .into_iter()
                .find(|x| x.id == id)
                .ok_or(RepoError::NotFound)?;
            to_string(&e)
        }
        _ => {
            to_string(&categories.into_iter()
                .filter(|e| ids.iter().any(|id| e.id == id.clone()))
                .collect::<Vec<Category>>())
        }
    }?;
    Ok(JSON(res))
}

#[derive(FromForm)]
struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    text: Option<String>,
    tags: Option<String>,
}

lazy_static! {
    static ref HASH_TAG_REGEX: Regex = Regex::new(r"#(?P<tag>\w+((-\w+)*)?)").unwrap();
}

fn extract_hash_tags(text: &str) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for cap in HASH_TAG_REGEX.captures_iter(text) {
        res.push(cap["tag"].into());
    }
    res
}

fn remove_hash_tags(text: &str) -> String {
    HASH_TAG_REGEX
        .replace_all(text, "")
        .into_owned()
        .replace("  ", " ")
        .trim()
        .into()
}

#[get("/search?<search>")]
fn get_search(db: State<DbPool>, search: SearchQuery) -> Result<json::SearchResult> {

    let entries = db.get()?.all_entries()?;

    let bbox = geo::extract_bbox(&search.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let mut entries: Vec<&Entry> = entries.iter().collect();

    if let Some(cat_str) = search.categories {
        let cat_ids = extract_ids(&cat_str);
        entries = entries
            .into_iter()
            .filter(&*filter::entries_by_category_ids(&cat_ids))
            .collect();
    }

    let mut tags = vec![];

    if let Some(ref txt) = search.text {
        tags = extract_hash_tags(txt);
    }

    if let Some(tags_str) = search.tags {
        for t in extract_ids(&tags_str) {
            tags.push(t);
        }
    }

    // search tags even without preceding #:
    if let Some(ref txt) = search.text {
        for t in to_words(txt){
            tags.push(t);
        }
    }

    let triples = db.get()?.all_triples()?;

    let text = match search.text {
        Some(txt) => remove_hash_tags(&txt),
        None => "".into()
    };

    let entries : Vec<&Entry> = entries
        .into_iter()
        .filter(&*filter::entries_by_tags_or_search_text(&text, &tags, &triples))
        .collect();

    let mut entries : Vec<Entry> = entries.into_iter().cloned().collect();

    let all_ratings = db.get()?.all_ratings()?;

    entries.sort_by_avg_rating(&all_ratings, &triples);

    let visible_results: Vec<_> = entries
        .iter()
        .filter(|x| x.in_bbox(&bbox))
        .map(|x| &x.id)
        .cloned()
        .collect();

    let invisible_results = entries
        .iter()
        .filter(|e| !visible_results.iter().any(|v| *v == e.id))
        .take(MAX_INVISIBLE_RESULTS)
        .map(|x| &x.id)
        .cloned()
        .collect::<Vec<_>>();

    Ok(JSON(json::SearchResult {
        visible: visible_results,
        invisible: invisible_results,
    }))
}

fn to_words(txt: &str) -> Vec<String> {
    txt.to_lowercase().split(' ').map(|x| x.to_string()).collect()
}

#[post("/login", format = "application/json", data = "<login>")]
fn login(db: State<DbPool>, mut cookies: Cookies, login: JSON<usecase::Login>) -> Result<()> {
    let id = usecase::login(&mut*db.get()?, login.into_inner())?;
    cookies.add_private(Cookie::new(COOKIE_USER_KEY, id));
    Ok(JSON(()))
}

#[post("/logout", format = "application/json")]
fn logout(mut cookies: Cookies) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Ok(JSON(()))
}

#[get("/users/<id>", format = "application/json")]
fn get_user(db: State<DbPool>, user: User, id: String) -> result::Result<JSON<json::User>,AppError> {
    let (username, email) = usecase::get_user(&mut*db.get()?, &user.0, &id)?;
    Ok(JSON(json::User{ username, email }))
}

#[post("/users", format = "application/json", data = "<u>")]
fn post_user(db: State<DbPool>, u: JSON<usecase::NewUser>) -> result::Result<(),AppError> {
    usecase::create_new_user(&mut*db.get()?, u.into_inner())?;
    Ok(())
}

#[post("/ratings", format = "application/json", data = "<u>")]
fn post_rating(db: State<DbPool>, u: JSON<usecase::RateEntry>) -> result::Result<(),AppError> {
    usecase::rate_entry(&mut*db.get()?, u.into_inner())?;
    Ok(())
}

#[get("/ratings/<id>")]
fn get_ratings(db: State<DbPool>, id: String)-> Result<Vec<json::Rating>>{
    let ratings = usecase::get_ratings(&*db.get()?,&extract_ids(&id))?;
    let r_ids : Vec<String> = ratings
        .iter()
        .map(|r|r.id.clone())
        .collect();
    let comments = usecase::get_comments_by_rating_ids(&*db.get()?,&r_ids)?;
    let triples = db.get()?.all_triples()?;
    let result = ratings
        .into_iter()
        .map(|x|json::Rating{
            id       : x.id.clone(),
            created  : x.created,
            title    : x.title,
            user     : usecase::get_user_id_for_rating_id(&triples,&x.id),
            value    : x.value,
            context  : x.context,
            source   : x.source.unwrap_or("".into()),
            comments : comments.get(&x.id).cloned().unwrap_or_else(|| vec![])
                .into_iter()
                .map(|c|json::Comment{
                    id: c.id.clone(),
                    created: c.created,
                    text: c.text,
                    user: usecase::get_user_id_for_comment_id(&triples,&c.id)
                 })
                .collect()
        })
        .collect();
    Ok(JSON(result))
}

#[get("/count/entries")]
fn get_count_entries(db: State<DbPool>) -> Result<usize> {
    let entries = db.get()?.all_entries()?;
    Ok(JSON(entries.len()))
}

#[get("/count/tags")]
fn get_count_tags(db: State<DbPool>) -> Result<usize> {
    let tags = usecase::get_tag_ids(&*db.get()?)?;
    Ok(JSON(tags.len()))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn rocket_instance<T: r2d2::ManageConnection>(cfg: Config, pool: Pool<T>) -> Rocket {

    rocket::custom(cfg, true)
        .manage(pool)
        .mount("/",
               routes![login,
                       logout,
                       get_entry,
                       post_entry,
                       post_user,
                       post_rating,
                       put_entry,
                       get_user,
                       get_categories,
                       get_tags,
                       get_ratings,
                       get_category,
                       get_search,
                       get_duplicates,
                       get_count_entries,
                       get_count_tags,
                       get_version])

}

pub fn run(db_url: &str, port: u16, enable_cors: bool) {

    if enable_cors {
        panic!("enable-cors is currently not available until\
        \nhttps://github.com/SergioBenitez/Rocket/pull/141\nis merged :(");
    }

    let cfg = Config::build(Environment::Production)
        .log_level(LoggingLevel::Normal)
        .address("127.0.0.1")
        .port(port)
        .finalize()
        .unwrap();

    let pool = neo4j::create_connection_pool(db_url).unwrap();

    rocket_instance(cfg, pool).launch();
}

impl<'r> Responder<'r> for AppError {
    fn respond_to(self, _: &rocket::Request) -> result::Result<Response<'r>, Status> {
        Err(match self {
            AppError::Business(ref err) => {
                match *err {
                    Error::Parameter(ref err) => {
                         match *err {
                            ParameterError::Credentials => Status::Unauthorized,
                            ParameterError::UserExists => <Status>::new(400, "UserExists")
                            _ => Status::BadRequest,
                         }
                    }
                    Error::Repo(ref err) => {
                        match *err {
                            RepoError::NotFound => Status::NotFound,
                            _ => Status::InternalServerError,
                        }
                    }
                    Error::Pwhash(_) => Status::InternalServerError
                }
            }

            _ => Status::InternalServerError,
        })
    }
}
