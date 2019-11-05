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
    routes![
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
        get_api,
        csv_export,
    ]
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
        for (e, _) in db.get_places(&ids)?.into_iter() {
            let r = db.load_ratings_of_entry(e.uid.as_ref())?;
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
        let entries = db.recently_changed_places(&params, &pagination)?;
        if with_ratings.unwrap_or(false) {
            let mut results = Vec::with_capacity(entries.len());
            for (e, _, _) in entries.into_iter() {
                let r = db.load_ratings_of_entry(e.uid.as_ref())?;
                results.push(json::Entry::from_entry_with_ratings(e, r));
            }
            results
        } else {
            entries
                .into_iter()
                .map(|(e, _, _)| json::Entry::from_entry_with_ratings(e, vec![]))
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
        db.most_popular_place_tags(&params, &pagination)?
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
        (db.get_places(&ids)?, db.all_places()?)
    };
    let results = usecases::find_duplicates(&entries, &all_entries);
    Ok(Json(
        results
            .into_iter()
            .map(|(uid1, uid2, dup)| (uid1.to_string(), uid2.to_string(), dup))
            .collect(),
    ))
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
    let login = login.into_inner();
    {
        let credentials = usecases::Credentials {
            email: &login.email,
            password: &login.password,
        };
        usecases::login_with_email(&*db.shared()?, &credentials)?;
    }
    cookies.add_private(
        Cookie::build(COOKIE_USER_KEY, login.email)
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

#[derive(Deserialize, Debug, Clone)]
struct ConfirmationToken {
    token: String,
}

#[post(
    "/confirm-email-address",
    format = "application/json",
    data = "<token>"
)]
fn confirm_email_address(db: sqlite::Connections, token: Json<ConfirmationToken>) -> Result<()> {
    let token = token.into_inner().token;
    usecases::confirm_email_address(&*db.exclusive()?, &token)?;
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
    let Login(email) = user;
    usecases::subscribe_to_bbox(&*db.exclusive()?, email, bbox)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(db: sqlite::Connections, user: Login) -> Result<()> {
    let Login(email) = user;
    usecases::unsubscribe_all_bboxes(&*db.exclusive()?, &email)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(
    db: sqlite::Connections,
    user: Login,
) -> Result<Vec<json::BboxSubscription>> {
    let Login(email) = user;
    let user_subscriptions = usecases::get_bbox_subscriptions(&*db.shared()?, &email)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.uid.into(),
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
    account: Option<Account>,
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    body: Json<usecases::NewPlace>,
) -> Result<String> {
    Ok(Json(
        flows::create_place(
            &connections,
            &mut search_engine,
            body.into_inner(),
            account.as_ref().map(|a| a.email()),
        )?
        .uid
        .to_string(),
    ))
}

#[put("/entries/<uid>", format = "application/json", data = "<data>")]
fn put_entry(
    account: Option<Account>,
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    uid: String,
    data: Json<usecases::UpdatePlace>,
) -> Result<String> {
    Ok(Json(
        flows::update_place(
            &connections,
            &mut search_engine,
            uid.into(),
            data.into_inner(),
            account.as_ref().map(|a| a.email()),
        )?
        .uid
        .into(),
    ))
}

#[get("/tags")]
fn get_tags(connections: sqlite::Connections) -> Result<Vec<String>> {
    let tags = connections.shared()?.all_tags()?;
    Ok(Json(tags.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
fn get_categories(connections: sqlite::Connections) -> Result<Vec<json::Category>> {
    let categories = connections
        .shared()?
        .all_categories()?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(categories))
}

#[get("/categories/<uids>")]
fn get_category(connections: sqlite::Connections, uids: String) -> Result<Vec<json::Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let uids = util::split_ids(&uids);
    if uids.is_empty() {
        return Ok(Json(vec![]));
    }
    let categories = connections
        .shared()?
        .all_categories()?
        .into_iter()
        .filter(|c| uids.iter().any(|uid| c.uid.as_str() == *uid))
        .map(Into::into)
        .collect();
    Ok(Json(categories))
}

#[derive(FromForm, Clone, Serialize)]
struct CsvExport {
    bbox: String,
}

#[get("/export/entries.csv?<export..>")]
fn csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    login: Login,
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

    let db = connections.shared()?;
    usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?;

    let entries_categories_and_ratings = {
        let all_categories: Vec<_> = db.all_categories()?;
        let limit = db.count_places()? + 100;
        usecases::search(&search_engine, req, limit)?
            .0
            .into_iter()
            .filter_map(|indexed_entry| {
                let IndexedPlace {
                    ref id,
                    ref ratings,
                    ..
                } = indexed_entry;
                if let Ok((mut place, _)) = db.get_place(id) {
                    let (tags, categories) = Category::split_from_tags(place.tags);
                    place.tags = tags;
                    let categories = all_categories
                        .iter()
                        .filter(|c1| categories.iter().any(|c2| c1.uid == c2.uid))
                        .cloned()
                        .collect::<Vec<Category>>();
                    Some((place, categories, ratings.total()))
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
