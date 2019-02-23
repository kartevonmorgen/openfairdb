use super::{guards::*, sqlite::DbConn, tantivy::SearchEngine, util};
use crate::{
    adapters::{self, json, user_communication},
    core::{
        prelude::*,
        usecases::{self, DuplicateType},
        util::geo,
    },
    infrastructure::error::AppError,
};

use csv;
use rocket::{
    self,
    http::{ContentType, Cookie, Cookies, Status},
    request::Form,
    response::{content::Content, Responder, Response},
    Route,
};
use rocket_contrib::json::Json;
use std::result;

mod count;
mod events;
pub mod geocoding;
mod ratings;
mod search;
#[cfg(test)]
pub mod tests;
mod users;

type Result<T> = result::Result<Json<T>, AppError>;

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
        events::post_event_with_token,
        events::get_event,
        events::get_events,
        events::get_events_with_token,
        events::put_event,
        events::put_event_with_token,
        events::delete_event,
        events::delete_event_with_token,
        users::post_user,
        ratings::post_rating,
        ratings::get_rating,
        users::get_user,
        users::delete_user,
        get_categories,
        get_category,
        get_tags,
        search::get_search,
        get_duplicates,
        count::get_count_entries,
        count::get_count_tags,
        get_version,
        csv_export,
        get_api
    ]
}

#[derive(Deserialize, Debug, Clone)]
struct UserId {
    u_id: String,
}

#[get("/entries/<ids>")]
fn get_entry(db: DbConn, ids: String) -> Result<Vec<json::Entry>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::extract_ids(&ids);
    let (entries, ratings) = {
        let db = db.pooled()?;
        let entries = usecases::get_entries(&*db, &ids)?;
        let ratings = usecases::get_ratings_by_entry_ids(&*db, &ids)?;
        (entries, ratings)
    };
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
    let entries = db.pooled()?.all_entries()?;
    let ids = usecases::find_duplicates(&entries);
    Ok(Json(ids))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[get("/server/api.yaml")]
fn get_api() -> Content<&'static str> {
    let data = include_str!("../../../../openapi.yaml");
    let c_type = ContentType::new("text", "yaml");
    Content(c_type, data)
}

#[post("/login", format = "application/json", data = "<login>")]
fn login(db: DbConn, mut cookies: Cookies, login: Json<usecases::Login>) -> Result<()> {
    let username = usecases::login(&mut *db.pooled()?, &login.into_inner())?;
    cookies.add_private(
        Cookie::build(COOKIE_USER_KEY, username)
            .same_site(rocket::http::SameSite::None)
            .finish(),
    );
    Ok(Json(()))
}

#[post("/logout", format = "application/json")]
fn logout(mut cookies: Cookies) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Ok(Json(()))
}

#[post("/confirm-email-address", format = "application/json", data = "<user>")]
fn confirm_email_address(db: DbConn, user: Json<UserId>) -> Result<()> {
    let u_id = user.into_inner().u_id;
    usecases::confirm_email_address(&mut *db.pooled()?, &u_id)?;
    Ok(Json(()))
}

#[post(
    "/subscribe-to-bbox",
    format = "application/json",
    data = "<coordinates>"
)]
fn subscribe_to_bbox(
    db: DbConn,
    user: Login,
    coordinates: Json<Vec<json::Coordinate>>,
) -> Result<()> {
    let coordinates: Vec<Coordinate> = coordinates
        .into_inner()
        .into_iter()
        .map(Coordinate::from)
        .collect();
    let Login(username) = user;
    usecases::subscribe_to_bbox(&coordinates, &username, &mut *db.pooled()?)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(db: DbConn, user: Login) -> Result<()> {
    let Login(username) = user;
    usecases::unsubscribe_all_bboxes_by_username(&mut *db.pooled()?, &username)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(db: DbConn, user: Login) -> Result<Vec<json::BboxSubscription>> {
    let Login(username) = user;
    let user_subscriptions = usecases::get_bbox_subscriptions(&username, &*db.pooled()?)?
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
fn post_entry(
    db: DbConn,
    mut search_engine: SearchEngine,
    e: Json<usecases::NewEntry>,
) -> Result<String> {
    let e = e.into_inner();
    let (id, email_addresses, all_categories) = {
        let mut db = db.pooled()?;
        let id = usecases::create_new_entry(&mut *db, Some(&mut search_engine), e.clone())?;
        let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
        let all_categories = db.all_categories()?;
        (id, email_addresses, all_categories)
    };
    util::notify_create_entry(&email_addresses, &e, &id, all_categories);
    Ok(Json(id))
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(
    db: DbConn,
    mut search_engine: SearchEngine,
    id: String,
    e: Json<usecases::UpdateEntry>,
) -> Result<String> {
    let e = e.into_inner();
    let (email_addresses, all_categories) = {
        let mut db = db.pooled()?;
        usecases::update_entry(&mut *db, Some(&mut search_engine), e.clone())?;
        let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
        let all_categories = db.all_categories()?;
        (email_addresses, all_categories)
    };
    util::notify_update_entry(&email_addresses, &e, all_categories);
    Ok(Json(id))
}

#[get("/tags")]
fn get_tags(db: DbConn) -> Result<Vec<String>> {
    let tags = db.pooled()?.all_tags()?;
    Ok(Json(tags.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
fn get_categories(db: DbConn) -> Result<Vec<Category>> {
    let categories = db.pooled()?.all_categories()?;
    Ok(Json(categories))
}

#[get("/categories/<ids>")]
fn get_category(db: DbConn, ids: String) -> Result<Vec<Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::extract_ids(&ids);
    let categories = db.pooled()?
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

// TODO: CSV export should only be permitted with a valid API key!
// https://github.com/slowtec/openfairdb/issues/147
#[get("/export/entries.csv?<export..>")]
fn csv_export<'a>(
    db: DbConn,
    search_engine: SearchEngine,
    export: Form<CsvExport>,
) -> result::Result<Content<String>, AppError> {
    let bbox = export
        .bbox
        .parse::<geo::MapBbox>()
        .map_err(|_| ParameterError::Bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let avg_ratings = match super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let req = usecases::SearchRequest {
        bbox,
        categories: Default::default(),
        text: Default::default(),
        tags: Default::default(),
        entry_ratings: &avg_ratings,
    };

    let (entries, all_categories) = {
        let db = db.pooled()?;
        let (entries, _) = usecases::search(&search_engine, &*db, req, None)?;
        let all_categories: Vec<_> = db.all_categories()?;
        (entries, all_categories)
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
                        ParameterError::Credentials | ParameterError::Unauthorized => {
                            Status::Unauthorized
                        }
                        ParameterError::UserExists => <Status>::new(400, "UserExists"),
                        ParameterError::EmailNotConfirmed => {
                            <Status>::new(403, "EmailNotConfirmed")
                        }
                        ParameterError::Forbidden | ParameterError::OwnedTag => Status::Forbidden,
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
