use rocket::{self, Rocket, State };
use rocket::logger::LoggingLevel;
use rocket_contrib::Json;
use rocket::response::{Response, Responder};
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use rocket::http::{Status, Cookie, Cookies};
use rocket::config::{Environment, Config};
use adapters::json;
use adapters::user_communication;
use entities::*;
use business::db::Db;
use business::error::{Error, RepoError, ParameterError};
use infrastructure::error::AppError;
use serde_json::ser::to_string;
use business::sort::SortByAverageRating;
use business::{usecase, filter, geo};
use business::filter::InBBox;
use business::duplicates::{self, DuplicateType};
use std::{result,thread};
use r2d2::{self, Pool};
use regex::Regex;
use super::mail;

static MAX_INVISIBLE_RESULTS : usize = 5;
static COOKIE_USER_KEY       : &str  = "user_id";

mod neo4j;
#[cfg(test)]
mod mockdb;
#[cfg(test)]
mod tests;

#[cfg(not(test))]
type DbPool = neo4j::ConnectionPool;
#[cfg(test)]
type DbPool = mockdb::ConnectionPool;

type Result<T> = result::Result<Json<T>, AppError>;

fn extract_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.to_owned())
        .filter(|id| id != "")
        .collect::<Vec<String>>()
}

#[derive(Deserialize, Debug, Clone)]
struct Login(String);

#[derive(Deserialize, Debug, Clone)]
struct UserId{
    u_id: String
}

impl<'a, 'r> FromRequest<'a, 'r> for Login {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Login, ()> {
        let user = request.cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| Login(id));
        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

fn send_mails(email_addresses: Vec<String>, subject: &str, body: &str) {
    debug!("sending emails to: {:?}", email_addresses);
    for email_address in email_addresses.clone() {
        let to = vec![email_address];
        match mail::create(&to, &subject, &body) {
            Ok(mail) => {
                thread::spawn(move ||{
                    if let Err(err) = mail::send(&mail) {
                        warn!("Could not send mail: {}", err);
                    }
                });
            }
            Err(e) => {
                warn!("could not create notification mail: {}", e);
            }
        }
    }
}

fn notify_create_entry(email_addresses: Vec<String>, e: &usecase::NewEntry, id: &str, all_categories: Vec<Category>) {
    let subject = String::from("Karte von Morgen - neuer Eintrag: ") + &e.title;
    let categories : Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::new_entry_email(e, id, categories);
    send_mails(email_addresses, &subject, &body);
}

fn notify_update_entry(email_addresses: Vec<String>, e: &usecase::UpdateEntry, all_categories: Vec<Category>) {
    let subject = String::from("Karte von Morgen - Eintrag verändert: ") + &e.title;
    let categories : Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::changed_entry_email(e, categories);
    send_mails(email_addresses, &subject, &body);
}

#[get("/entries/<ids>")]
fn get_entry(db: State<DbPool>, ids: String) -> Result<Vec<json::Entry>> {
    let ids = extract_ids(&ids);
    let entries = usecase::get_entries(&*db.get()?, &ids)?;
    let tags = usecase::get_tags_by_entry_ids(&*db.get()?, &ids)?;
    let ratings = usecase::get_ratings_by_entry_ids(&*db.get()?, &ids)?;
    Ok(Json(entries
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
    Ok(Json(ids))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(db: State<DbPool>, e: Json<usecase::NewEntry>) -> Result<String> {
    let e = e.into_inner();
    let id = usecase::create_new_entry(&mut *db.get()?, e.clone())?;
    let email_addresses = usecase::email_addresses_to_notify(&e.lat, &e.lng, &mut *db.get()?);
    let all_categories = db.get()?.all_categories()?;
    notify_create_entry(email_addresses, &e, &id, all_categories);
    Ok(Json(id))
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(db: State<DbPool>, id: String, e: Json<usecase::UpdateEntry>) -> Result<String> {
    let e = e.into_inner();
    usecase::update_entry(&mut *db.get()?, e.clone())?;
    let email_addresses = usecase::email_addresses_to_notify(&e.lat, &e.lng, &mut *db.get()?);
    let all_categories = db.get()?.all_categories()?;
    notify_update_entry(email_addresses, &e, all_categories);
    Ok(Json(id))
}

#[get("/tags")]
fn get_tags(db: State<DbPool>) -> Result<Vec<String>> {
    let tags = usecase::get_tag_ids(&*db.get()?)?;
    Ok(Json(tags))
}

#[get("/categories")]
fn get_categories(db: State<DbPool>) -> Result<Vec<Category>> {
    let categories = db.get()?.all_categories()?;
    Ok(Json(categories))
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
    Ok(Json(res))
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

    Ok(Json(json::SearchResult {
        visible: visible_results,
        invisible: invisible_results,
    }))
}

fn to_words(txt: &str) -> Vec<String> {
    txt.to_lowercase().split(' ').map(|x| x.to_string()).collect()
}

#[post("/login", format = "application/json", data = "<login>")]
fn login(db: State<DbPool>, mut cookies: Cookies, login: Json<usecase::Login>) -> Result<()> {
    let id = usecase::login(&mut*db.get()?, login.into_inner())?;
    cookies.add_private(Cookie::new(COOKIE_USER_KEY, id));
    Ok(Json(()))
}

#[post("/logout", format = "application/json")]
fn logout(mut cookies: Cookies) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Ok(Json(()))
}

#[post("/confirm-email-address", format = "application/json", data = "<user>")]
fn confirm_email_address(user : Json<UserId>, db: State<DbPool>) -> Result<()>{
    let u_id = user.into_inner().u_id;
    let u = db.get()?.confirm_email_address(&u_id)?;
    if u.id == u_id {
        Ok(Json(()))
    } else {
        Err(AppError::Business(Error::Repo(RepoError::NotFound)))
    }
}

#[post("/subscribe-to-bbox", format = "application/json", data = "<coordinates>")]
fn subscribe_to_bbox(user: Login, coordinates: Json<Vec<Coordinate>>, db: State<DbPool>) -> Result<()> {
    let coordinates = coordinates.into_inner();
    let Login(username) = user;
    usecase::subscribe_to_bbox(&coordinates, &username, &mut*db.get()?)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(user: Login, db: State<DbPool>) -> Result<()> {
    let Login(username) = user;
    usecase::unsubscribe_all_bboxes(&username, &mut*db.get()?)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(db: State<DbPool>, user: Login) -> Result<Vec<json::BboxSubscription>> {
    let Login(username) = user;
    let user_subscriptions = usecase::get_bbox_subscriptions(&username, &*db.get()?)?
        .into_iter()
        .map(|s| json::BboxSubscription{
            id: s.id,
            south_west_lat: s.south_west_lat,
            south_west_lng: s.south_west_lng,
            north_east_lat: s.north_east_lat,
            north_east_lng: s.north_east_lng,
        })
        .collect();
    Ok(Json(user_subscriptions))
}

#[get("/users/<username>", format = "application/json")]
fn get_user(db: State<DbPool>, user: Login, username: String) -> Result<json::User> {
    let (u_id, email) = usecase::get_user(&mut*db.get()?, &user.0, &username)?;
    Ok(Json(json::User{ u_id, email }))
}

#[post("/users", format = "application/json", data = "<u>")]
fn post_user(db: State<DbPool>, u: Json<usecase::NewUser>) -> Result<()> {
    let new_user = u.into_inner();
    usecase::create_new_user(&mut*db.get()?, new_user.clone())?;
    let user = db.get()?.get_user(&new_user.username)?;
    let subject = "Karte von Morgen: bitte bestätige deine Email-Adresse";
    let body = user_communication::email_confirmation_email(&user.id);
    send_mails(vec![user.email.clone()], &subject, &body);
    Ok(Json(()))
}

#[delete("/users/<u_id>")]
fn delete_user(db: State<DbPool>, user: Login, u_id: String) -> Result<()> {
    usecase::delete_user(&mut*db.get()?, &user.0, &u_id)?;
    Ok(Json(()))
}

#[post("/ratings", format = "application/json", data = "<u>")]
fn post_rating(db: State<DbPool>, u: Json<usecase::RateEntry>) -> Result<()> {
    usecase::rate_entry(&mut*db.get()?, u.into_inner())?;
    Ok(Json(()))
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
    Ok(Json(result))
}

#[get("/count/entries")]
fn get_count_entries(db: State<DbPool>) -> Result<usize> {
    let entries = db.get()?.all_entries()?;
    Ok(Json(entries.len()))
}

#[get("/count/tags")]
fn get_count_tags(db: State<DbPool>) -> Result<usize> {
    let tags = usecase::get_tag_ids(&*db.get()?)?;
    Ok(Json(tags.len()))
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
                       // send_confirmation_email,
                       delete_user,
                       confirm_email_address,
                       subscribe_to_bbox,
                       get_bbox_subscriptions,
                       unsubscribe_all_bboxes,
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
                            ParameterError::UserExists => <Status>::new(400, "UserExists"),
                            ParameterError::EmailNotConfirmed => <Status>::new(403, "EmailNotConfirmed"),
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
