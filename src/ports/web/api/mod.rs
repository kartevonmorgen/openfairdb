use super::guards::*;

use crate::{
    adapters::{self, json},
    core::{
        prelude::*,
        usecases::{self, DuplicateType},
        util::{self, geo},
    },
    infrastructure::{
        db::{sqlite, tantivy},
        error::AppError,
        flows::prelude as flows,
        notify,
    },
};

#[cfg(feature = "export")]
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
pub mod events;
pub mod geocoding;
mod ratings;
mod search;
#[cfg(test)]
pub mod tests;
mod users;

type Result<T> = result::Result<Json<T>, AppError>;

pub fn routes() -> Vec<Route> {
    let mut routes = routes![
        login,
        logout,
        confirm_email_address,
        subscribe_to_bbox,
        get_bbox_subscriptions,
        unsubscribe_all_bboxes,
        get_entry,
        get_entries_recently_changed,
        get_entries_most_popular_tags,
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
        users::post_request_password_reset,
        users::post_reset_password,
        users::post_user,
        ratings::post_rating,
        ratings::load_rating,
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
        get_api
    ];
    if cfg!(feature = "export") {
        routes.append(&mut routes![csv_export]);
    }
    routes
}

#[derive(Deserialize, Debug, Clone)]
struct UserId {
    u_id: String,
}

#[get("/entries/<ids>")]
fn get_entry(db: sqlite::Connections, ids: String) -> Result<Vec<json::Entry>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let results = {
        let mut results = Vec::with_capacity(ids.len());
        let db = db.shared()?;
        for e in db.get_entries(&ids)?.into_iter() {
            let r = db.load_ratings_of_entry(&e.id)?;
            results.push(json::Entry::from_entry_with_ratings(e, r));
        }
        results
    };
    Ok(Json(results))
}

// Limit the total number of recently changed entries to avoid cloning
// the whole database!!

const ENTRIES_RECECENTLY_CHANGED_MAX_COUNT: u64 = 1000;

const ENTRIES_RECECENTLY_CHANGED_MAX_AGE_IN_DAYS: i64 = 100;

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

#[get("/entries/recently-changed?<since>&<until>&<with_ratings>&<offset>&<limit>")]
fn get_entries_recently_changed(
    db: sqlite::Connections,
    since: Option<i64>,
    until: Option<i64>,
    with_ratings: Option<bool>,
    offset: Option<u64>,
    mut limit: Option<u64>,
) -> Result<Vec<json::Entry>> {
    let since_min =
        i64::from(Timestamp::now()) - ENTRIES_RECECENTLY_CHANGED_MAX_AGE_IN_DAYS * SECONDS_PER_DAY;
    let since = if let Some(since) = since {
        if since < since_min {
            log::warn!(
                "Maximum available age of recently changed entries exceeded: {} < {}",
                since,
                since_min
            );
            Some(since_min)
        } else {
            Some(since)
        }
    } else {
        Some(since_min)
    }
    .map(Into::into);
    debug_assert!(since.is_some());
    let mut total_count = 0;
    if let Some(offset) = offset {
        total_count += offset;
    }
    total_count += limit.unwrap_or(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT);
    if total_count > ENTRIES_RECECENTLY_CHANGED_MAX_COUNT {
        log::warn!(
            "Maximum available number of recently changed entries exceeded: {} > {}",
            total_count,
            ENTRIES_RECECENTLY_CHANGED_MAX_COUNT
        );
        if let Some(offset) = offset {
            limit = Some(
                ENTRIES_RECECENTLY_CHANGED_MAX_COUNT
                    - offset.min(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT),
            );
        } else {
            limit = Some(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT);
        }
    } else {
        limit = Some(limit.unwrap_or(ENTRIES_RECECENTLY_CHANGED_MAX_COUNT - offset.unwrap_or(0)));
    }
    debug_assert!(limit.is_some());
    let params = RecentlyChangedEntriesParams {
        since,
        until: until.map(Into::into),
    };
    let pagination = Pagination { offset, limit };
    let results = {
        let db = db.shared()?;
        let entries = db.recently_changed_entries(&params, &pagination)?;
        if with_ratings.unwrap_or(false) {
            let mut results = Vec::with_capacity(entries.len());
            for e in entries.into_iter() {
                let r = db.load_ratings_of_entry(&e.id)?;
                results.push(json::Entry::from_entry_with_ratings(e, r));
            }
            results
        } else {
            entries
                .into_iter()
                .map(|e| json::Entry::from_entry_with_ratings(e, vec![]))
                .collect()
        }
    };
    Ok(Json(results))
}

const ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX: u64 = 1000;

#[get("/entries/most-popular-tags?<min_count>&<max_count>&<offset>&<limit>")]
pub fn get_entries_most_popular_tags(
    db: sqlite::Connections,
    min_count: Option<u64>,
    max_count: Option<u64>,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<json::TagFrequency>> {
    let params = MostPopularTagsParams {
        min_count,
        max_count,
    };
    let limit = Some(
        limit
            .unwrap_or(ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX)
            .min(ENTRIES_MOST_POPULAR_TAGS_PAGINATION_LIMIT_MAX),
    );
    let pagination = Pagination { offset, limit };
    let results = {
        let db = db.shared()?;
        db.most_popular_entry_tags(&params, &pagination)?
    };
    Ok(Json(results.into_iter().map(Into::into).collect()))
}

#[get("/duplicates/<ids>")]
fn get_duplicates(
    db: sqlite::Connections,
    ids: String,
) -> Result<Vec<(String, String, DuplicateType)>> {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let (entries, all_entries) = {
        let db = db.shared()?;
        (db.get_entries(&ids)?, db.all_entries()?)
    };
    let results = usecases::find_duplicates(&entries, &all_entries);
    Ok(Json(results))
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
fn login(
    db: sqlite::Connections,
    mut cookies: Cookies,
    login: Json<usecases::Login>,
) -> Result<()> {
    //TODO: login with email
    let username = usecases::login_with_username(&*db.shared()?, &login.into_inner())?;
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
fn confirm_email_address(db: sqlite::Connections, user: Json<UserId>) -> Result<()> {
    let u_id = user.into_inner().u_id;
    usecases::confirm_email_address(&*db.exclusive()?, &u_id)?;
    Ok(Json(()))
}

#[post(
    "/subscribe-to-bbox",
    format = "application/json",
    data = "<coordinates>"
)]
fn subscribe_to_bbox(
    db: sqlite::Connections,
    user: Login,
    coordinates: Json<Vec<json::Coordinate>>,
) -> Result<()> {
    let sw_ne: Vec<_> = coordinates
        .into_inner()
        .into_iter()
        .map(MapPoint::from)
        .collect();
    if sw_ne.len() != 2 {
        return Err(Error::Parameter(ParameterError::Bbox).into());
    }
    let bbox = geo::MapBbox::new(sw_ne[0], sw_ne[1]);
    let Login(username) = user;
    usecases::subscribe_to_bbox(bbox, &username, &mut *db.exclusive()?)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(db: sqlite::Connections, user: Login) -> Result<()> {
    let Login(username) = user;
    usecases::unsubscribe_all_bboxes_by_username(&mut *db.exclusive()?, &username)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(
    db: sqlite::Connections,
    user: Login,
) -> Result<Vec<json::BboxSubscription>> {
    let Login(username) = user;
    let user_subscriptions = usecases::get_bbox_subscriptions(&username, &*db.shared()?)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.id,
            south_west_lat: s.bbox.south_west().lat().to_deg(),
            south_west_lng: s.bbox.south_west().lng().to_deg(),
            north_east_lat: s.bbox.north_east().lat().to_deg(),
            north_east_lng: s.bbox.north_east().lng().to_deg(),
        })
        .collect();
    Ok(Json(user_subscriptions))
}

#[post("/entries", format = "application/json", data = "<body>")]
fn post_entry(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    body: Json<usecases::NewEntry>,
) -> Result<String> {
    Ok(Json(flows::create_entry(
        &connections,
        &mut search_engine,
        body.into_inner(),
    )?))
}

#[put("/entries/<id>", format = "application/json", data = "<data>")]
fn put_entry(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    id: String,
    data: Json<usecases::UpdateEntry>,
) -> Result<String> {
    Ok(Json(
        flows::update_entry(&connections, &mut search_engine, id, data.into_inner())?.id,
    ))
}

#[get("/tags")]
fn get_tags(connections: sqlite::Connections) -> Result<Vec<String>> {
    let tags = connections.shared()?.all_tags()?;
    Ok(Json(tags.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
fn get_categories(connections: sqlite::Connections) -> Result<Vec<Category>> {
    let categories = connections.shared()?.all_categories()?;
    Ok(Json(categories))
}

#[get("/categories/<ids>")]
fn get_category(connections: sqlite::Connections, ids: String) -> Result<Vec<Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let categories = connections
        .shared()?
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
// NOTE: As a temporary workaround the CSV export must be enabled
// explicitly with a build feature flag.
#[get("/export/entries.csv?<export..>")]
fn csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    export: Form<CsvExport>,
) -> result::Result<Content<String>, AppError> {
    let bbox = export
        .bbox
        .parse::<geo::MapBbox>()
        .map_err(|_| ParameterError::Bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let req = usecases::SearchRequest {
        bbox,
        ids: vec![],
        categories: vec![],
        hash_tags: vec![],
        text: None,
    };

    let entries_categories_and_ratings = {
        let db = connections.shared()?;
        let all_categories: Vec<_> = db.all_categories()?;
        let limit = db.count_entries()? + 100;
        usecases::search(&search_engine, req, limit)?
            .0
            .into_iter()
            .filter_map(|indexed_entry| {
                let IndexedEntry {
                    ref id,
                    ref ratings,
                    ..
                } = indexed_entry;
                if let Ok(entry) = db.get_entry(id) {
                    let categories = all_categories
                        .iter()
                        .filter(|c1| entry.categories.iter().any(|c2| *c2 == c1.id))
                        .cloned()
                        .collect::<Vec<Category>>();
                    Some((entry, categories, ratings.total()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };

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
