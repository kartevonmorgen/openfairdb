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

type StatusResult = result::Result<Status, AppError>;

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
        get_place,
        get_place_history,
        post_places_review,
        post_entry,
        put_entry,
        events::post_event,
        events::post_event_with_token,
        events::get_event,
        events::get_events_chronologically,
        events::get_events_with_token,
        events::put_event,
        events::put_event_with_token,
        events::post_events_archive,
        events::delete_event,
        events::delete_event_with_token,
        events::csv_export_with_token,
        events::csv_export_without_token,
        users::post_request_password_reset,
        users::post_reset_password,
        users::post_user,
        ratings::post_rating,
        ratings::load_rating,
        users::get_user,
        users::get_current_user,
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
        entries_csv_export_with_token,
        entries_csv_export_without_token,
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
        for (place, _) in db.get_places(&ids)?.into_iter() {
            let r = db.load_ratings_of_place(place.id.as_ref())?;
            results.push(json::entry_from_place_with_ratings(place, r));
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
    since: Option<i64>, // in seconds
    until: Option<i64>, // in seconds
    with_ratings: Option<bool>,
    offset: Option<u64>,
    mut limit: Option<u64>,
) -> Result<Vec<json::Entry>> {
    let since_min = Timestamp::now().into_seconds()
        - ENTRIES_RECECENTLY_CHANGED_MAX_AGE_IN_DAYS * SECONDS_PER_DAY;
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
    };
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
    // Conversion from seconds (external) to milliseconds (internal)
    let params = RecentlyChangedEntriesParams {
        since: since.map(TimestampMs::from_seconds),
        until: until.map(TimestampMs::from_seconds),
    };
    let pagination = Pagination { offset, limit };
    let results = {
        let db = db.shared()?;
        let entries = db.recently_changed_places(&params, &pagination)?;
        if with_ratings.unwrap_or(false) {
            let mut results = Vec::with_capacity(entries.len());
            for (place, _, _) in entries.into_iter() {
                let r = db.load_ratings_of_place(place.id.as_ref())?;
                results.push(json::entry_from_place_with_ratings(place, r));
            }
            results
        } else {
            entries
                .into_iter()
                .map(|(place, _, _)| json::entry_from_place_with_ratings(place, vec![]))
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
        db.most_popular_place_revision_tags(&params, &pagination)?
    };
    Ok(Json(results.into_iter().map(Into::into).collect()))
}

#[get("/places/<id>")]
pub fn get_place(
    db: sqlite::Connections,
    id: String,
) -> Result<(json::PlaceRoot, json::PlaceRevision, json::ReviewStatus)> {
    let (place, status) = {
        let db = db.shared()?;
        db.get_place(&id)?
    };
    let (place_root, place_revision) = place.into();
    Ok(Json((
        place_root.into(),
        place_revision.into(),
        status.into(),
    )))
}

#[get("/places/<id>/history")]
pub fn get_place_history(
    db: sqlite::Connections,
    login: Login,
    id: String,
) -> Result<json::PlaceHistory> {
    let place_history = {
        let db = db.shared()?;

        // The history contains e-mail addresses of registered users
        // and is only permitted for scouts and admins!
        usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?;

        db.get_place_history(&id)?
    };
    Ok(Json(place_history.into()))
}

#[post("/places/<ids>/review", data = "<review>")]
pub fn post_places_review(
    login: Login,
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    ids: String,
    review: Json<json::Review>,
) -> Result<()> {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyIdList).into());
    }
    let reviewer_email = {
        let db = db.shared()?;
        // Only scouts and admins are entitled to review places
        usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?.email
    };
    let json::Review { status, comment } = review.into_inner();
    // TODO: Record context information
    let context = None;
    let review = usecases::Review {
        context,
        reviewer_email: reviewer_email.into(),
        status: status.into(),
        comment,
    };
    let update_count = flows::review_places(&db, &mut search_engine, &ids, review)?;
    if update_count < ids.len() {
        log::warn!(
            "Applied review to only {} of {} place(s): {:?}",
            update_count,
            ids.len(),
            ids
        );
    }
    Ok(Json(()))
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
            .map(|(id1, id2, dup)| (id1.to_string(), id2.to_string(), dup))
            .collect(),
    ))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[get("/server/openapi.yaml")]
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
            id: s.id.into(),
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
        .id
        .to_string(),
    ))
}

#[put("/entries/<id>", format = "application/json", data = "<data>")]
fn put_entry(
    account: Option<Account>,
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    id: String,
    data: Json<usecases::UpdatePlace>,
) -> Result<String> {
    Ok(Json(
        flows::update_place(
            &connections,
            &mut search_engine,
            id.into(),
            data.into_inner(),
            account.as_ref().map(|a| a.email()),
        )?
        .id
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

#[get("/categories/<ids>")]
fn get_category(connections: sqlite::Connections, ids: String) -> Result<Vec<json::Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let uids = util::split_ids(&ids);
    if uids.is_empty() {
        return Ok(Json(vec![]));
    }
    let categories = connections
        .shared()?
        .all_categories()?
        .into_iter()
        .filter(|c| uids.iter().any(|id| c.id.as_str() == *id))
        .map(Into::into)
        .collect();
    Ok(Json(categories))
}

#[get("/export/entries.csv?<query..>")]
fn entries_csv_export_with_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    token: Bearer,
    login: Login,
    query: Form<search::SearchQuery>,
) -> result::Result<Content<String>, AppError> {
    let organization =
        usecases::authorize_organization_by_token(&*connections.shared()?, &token.0)?;
    entries_csv_export(
        connections,
        search_engine,
        Some(organization),
        login,
        query.into_inner(),
    )
}

#[get("/export/entries.csv?<query..>", rank = 2)]
fn entries_csv_export_without_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    login: Login,
    query: Form<search::SearchQuery>,
) -> result::Result<Content<String>, AppError> {
    entries_csv_export(connections, search_engine, None, login, query.into_inner())
}

fn entries_csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    org: Option<Organization>,
    login: Login,
    query: search::SearchQuery,
) -> result::Result<Content<String>, AppError> {
    let owned_tags = org.map(|org| org.owned_tags).unwrap_or_default();

    let db = connections.shared()?;
    let user = usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?;

    let (req, limit) = search::parse_search_query(&query)?;
    let limit = if let Some(limit) = limit {
        // Limited
        limit
    } else {
        // Unlimited
        db.count_places()? + 100
    };

    let entries_categories_and_ratings = {
        let all_categories: Vec<_> = db.all_categories()?;
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
                        .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
                        .cloned()
                        .collect::<Vec<Category>>();
                    Some((place, categories, ratings.total()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    // Release the database connection asap
    drop(db);

    let records: Vec<_> = entries_categories_and_ratings
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
