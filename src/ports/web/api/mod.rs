use super::{sqlite::DbConn, util};
use crate::adapters::{self, json, user_communication};
use crate::core::{
    prelude::*,
    usecases::{self, DuplicateType},
    util::geo,
};
use crate::infrastructure::error::AppError;
use csv;
use rocket::{
    self,
    http::{ContentType, Cookie, Cookies, Status},
    request::{self, Form, FromRequest, Request},
    response::{content::Content, Responder, Response},
    Outcome, Route,
};
use rocket_contrib::json::Json;
use std::result;

mod count;
mod events;
mod ratings;
#[cfg(test)]
pub mod tests;
mod users;

type Result<T> = result::Result<Json<T>, AppError>;

const COOKIE_USER_KEY: &str = "user_id";

impl<'a, 'r> FromRequest<'a, 'r> for Login {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Login, ()> {
        let user = request
            .cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Login);
        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        login,
        logout,
        confirm_email_address,
        subscribe_to_bbox,
        get_bbox_subscriptions,
        unsubscribe_all_bboxes,
        get_entry,
        post_entry,
        put_entry,
        events::post_event,
        events::get_event,
        users::post_user,
        ratings::post_rating,
        ratings::get_rating,
        users::get_user,
        users::delete_user,
        get_categories,
        get_category,
        get_tags,
        get_search,
        get_duplicates,
        count::get_count_entries,
        count::get_count_tags,
        get_version,
        csv_export
    ]
}

#[derive(FromForm, Clone)]
struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    text: Option<String>,
    tags: Option<String>,
}

#[get("/search?<search..>")]
fn get_search(db: DbConn, search: Form<SearchQuery>) -> Result<json::SearchResponse> {
    let bbox = geo::extract_bbox(&search.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let categories = match search.categories {
        Some(ref cat_str) => Some(util::extract_ids(&cat_str)),
        None => None,
    };

    let mut tags = vec![];

    if let Some(ref txt) = search.text {
        tags = util::extract_hash_tags(txt);
    }

    if let Some(ref tags_str) = search.tags {
        for t in util::extract_ids(tags_str) {
            tags.push(t);
        }
    }

    let text = match search.text {
        Some(ref txt) => util::remove_hash_tags(txt),
        None => "".into(),
    };

    let avg_ratings = match super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let req = usecases::SearchRequest {
        bbox,
        categories,
        text,
        tags,
        entry_ratings: &*avg_ratings,
    };

    let (visible, invisible) = usecases::search(&*db, &req)?;

    let visible = visible
        .into_iter()
        .map(json::EntryIdWithCoordinates::from)
        .collect();

    let invisible = invisible
        .into_iter()
        .map(json::EntryIdWithCoordinates::from)
        .collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}

#[derive(Deserialize, Debug, Clone)]
pub struct Login(String);

#[derive(Deserialize, Debug, Clone)]
struct UserId {
    u_id: String,
}

#[get("/entries/<ids>")]
fn get_entry(db: DbConn, ids: String) -> Result<Vec<json::Entry>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::extract_ids(&ids);
    let entries = usecases::get_entries(&*db, &ids)?;
    let ratings = usecases::get_ratings_by_entry_ids(&*db, &ids)?;
    Ok(Json(
        entries
            .into_iter()
            .map(|e| {
                let r = ratings.get(&e.id).cloned().unwrap_or_else(|| vec![]);
                json::Entry::from_entry_with_ratings(e, r)
            })
            .collect::<Vec<json::Entry>>(),
    ))
}

#[get("/duplicates")]
fn get_duplicates(db: DbConn) -> Result<Vec<(String, String, DuplicateType)>> {
    let entries = db.all_entries()?;
    let ids = usecases::find_duplicates(&entries);
    Ok(Json(ids))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[post("/login", format = "application/json", data = "<login>")]
fn login(mut db: DbConn, mut cookies: Cookies, login: Json<usecases::Login>) -> Result<()> {
    let username = usecases::login(&mut *db, &login.into_inner())?;
    cookies.add_private(Cookie::new(COOKIE_USER_KEY, username));
    Ok(Json(()))
}

#[post("/logout", format = "application/json")]
fn logout(mut cookies: Cookies) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Ok(Json(()))
}

#[post("/confirm-email-address", format = "application/json", data = "<user>")]
fn confirm_email_address(mut db: DbConn, user: Json<UserId>) -> Result<()> {
    let u_id = user.into_inner().u_id;
    usecases::confirm_email_address(&mut *db, &u_id)?;
    Ok(Json(()))
}

#[post(
    "/subscribe-to-bbox",
    format = "application/json",
    data = "<coordinates>"
)]
fn subscribe_to_bbox(
    mut db: DbConn,
    user: Login,
    coordinates: Json<Vec<json::Coordinate>>,
) -> Result<()> {
    let coordinates: Vec<Coordinate> = coordinates
        .into_inner()
        .into_iter()
        .map(Coordinate::from)
        .collect();
    let Login(username) = user;
    usecases::subscribe_to_bbox(&coordinates, &username, &mut *db)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(mut db: DbConn, user: Login) -> Result<()> {
    let Login(username) = user;
    usecases::unsubscribe_all_bboxes_by_username(&mut *db, &username)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(db: DbConn, user: Login) -> Result<Vec<json::BboxSubscription>> {
    let Login(username) = user;
    let user_subscriptions = usecases::get_bbox_subscriptions(&username, &*db)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.id,
            south_west_lat: s.bbox.south_west.lat,
            south_west_lng: s.bbox.south_west.lng,
            north_east_lat: s.bbox.north_east.lat,
            north_east_lng: s.bbox.north_east.lng,
        })
        .collect();
    Ok(Json(user_subscriptions))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(mut db: DbConn, e: Json<usecases::NewEntry>) -> Result<String> {
    let e = e.into_inner();
    let id = usecases::create_new_entry(&mut *db, e.clone())?;
    let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
    let all_categories = db.all_categories()?;
    util::notify_create_entry(&email_addresses, &e, &id, all_categories);
    Ok(Json(id))
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(mut db: DbConn, id: String, e: Json<usecases::UpdateEntry>) -> Result<String> {
    let e = e.into_inner();
    usecases::update_entry(&mut *db, e.clone())?;
    let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
    let all_categories = db.all_categories()?;
    util::notify_update_entry(&email_addresses, &e, all_categories);
    Ok(Json(id))
}

#[get("/tags")]
fn get_tags(db: DbConn) -> Result<Vec<String>> {
    Ok(Json(db.all_tags()?.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
fn get_categories(db: DbConn) -> Result<Vec<Category>> {
    let categories = db.all_categories()?;
    Ok(Json(categories))
}

#[get("/categories/<ids>")]
fn get_category(db: DbConn, ids: String) -> Result<Vec<Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::extract_ids(&ids);
    let categories = db
        .all_categories()?
        .into_iter()
        .filter(|c| ids.iter().any(|id| &c.id == id))
        .collect::<Vec<Category>>();
    Ok(Json(categories))
}

#[derive(FromForm, Clone, Serialize)]
struct CsvExport {
    bbox: String,
}

#[get("/export/entries.csv?<export..>")]
fn csv_export<'a>(
    db: DbConn,
    export: Form<CsvExport>,
) -> result::Result<Content<String>, AppError> {
    let bbox = geo::extract_bbox(&export.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let entries: Vec<_> = db.get_entries_by_bbox(&bbox)?;
    let all_categories: Vec<_> = db.all_categories()?;
    let avg_ratings = match super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let entries_categories_and_ratings = entries
        .into_iter()
        .map(|e| {
            let categories = all_categories
                .iter()
                .filter(|c1| e.categories.iter().any(|c2| *c2 == c1.id))
                .cloned()
                .collect::<Vec<Category>>();
            let avg_rating = *avg_ratings.get(&e.id).unwrap_or_else(|| &0.0);
            (e, categories, avg_rating)
        })
        .collect::<Vec<_>>();

    let records: Vec<adapters::csv::CsvRecord> = entries_categories_and_ratings
        .into_iter()
        .map(adapters::csv::CsvRecord::from)
        .collect();

    let buff: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buff);

    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(Content(ContentType::CSV, data))
}

impl<'r> Responder<'r> for AppError {
    fn respond_to(self, _: &rocket::Request) -> result::Result<Response<'r>, Status> {
        if let AppError::Business(ref err) = self {
            match *err {
                Error::Parameter(ref err) => {
                    return Err(match *err {
                        ParameterError::Credentials => Status::Unauthorized,
                        ParameterError::UserExists => <Status>::new(400, "UserExists"),
                        ParameterError::EmailNotConfirmed => {
                            <Status>::new(403, "EmailNotConfirmed")
                        }
                        ParameterError::Forbidden => Status::Forbidden,
                        _ => Status::BadRequest,
                    });
                }
                Error::Repo(ref err) => {
                    if let RepoError::NotFound = *err {
                        return Err(Status::NotFound);
                    }
                }
                _ => {}
            }
        }
        error!("Error: {}", self);
        Err(Status::InternalServerError)
    }
}
