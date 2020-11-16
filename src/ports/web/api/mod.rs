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
    },
    ports::web::{jwt, notify::*},
};
use rocket::{
    self,
    http::{ContentType, Cookie, Cookies, Status},
    request::Form,
    response::{content::Content, Responder, Response},
    Route, State,
};
use rocket_contrib::json::Json;
use std::result;

pub mod captcha;
mod count;
mod entries;
pub mod events;
mod places;
mod ratings;
mod search;
#[cfg(test)]
pub mod tests;
mod users;

type Result<T> = result::Result<Json<T>, AppError>;
type StatusResult = result::Result<Status, AppError>;

pub fn routes() -> Vec<Route> {
    routes![
        post_login,
        post_logout,
        confirm_email_address,
        subscribe_to_bbox,
        get_bbox_subscriptions,
        unsubscribe_all_bboxes,
        entries::get_entry,
        entries::get_entries_recently_changed,
        entries::get_entries_most_popular_tags,
        entries::post_entry,
        entries::put_entry,
        get_place,
        get_place_history,
        get_place_history_revision,
        post_places_review,
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
        events::csv_export,
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
        search::post_search_duplicates,
        count::get_count_entries,
        count::get_count_tags,
        get_version,
        get_api,
        entries_csv_export,
        places::count_pending_clearances,
        places::list_pending_clearances,
        places::update_pending_clearances,
        captcha::post_captcha,
        captcha::get_captcha,
        captcha::post_captcha_verify,
    ]
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

#[get("/places/<id>/history/<revision>")]
pub fn get_place_history_revision(
    db: sqlite::Connections,
    account: Option<Account>,
    bearer: Option<Bearer>,
    id: String,
    revision: RevisionValue,
) -> Result<json::PlaceHistory> {
    let place_history = {
        let db = db.shared()?;

        // The history contains e-mail addresses of registered users
        // is only permitted for scouts and admins or organizations!
        if let Some(account) = account {
            usecases::authorize_user_by_email(&*db, account.email(), Role::Scout)?;
        } else if let Some(bearer) = bearer {
            let api_token = bearer.0;
            usecases::authorize_organization_by_api_token(&*db, &api_token)?;
        } else {
            return Err(Error::Parameter(ParameterError::Unauthorized).into());
        }

        db.get_place_history(&id, Some(revision.into()))?
    };
    Ok(Json(place_history.into()))
}

#[get("/places/<id>/history", rank = 2)]
pub fn get_place_history(
    db: sqlite::Connections,
    account: Option<Account>,
    bearer: Option<Bearer>,
    id: String,
) -> Result<json::PlaceHistory> {
    let place_history = {
        let db = db.shared()?;

        // The history contains e-mail addresses of registered users
        // is only permitted for scouts and admins or for organizations!
        if let Some(account) = account {
            usecases::authorize_user_by_email(&*db, account.email(), Role::Scout)?;
        } else if let Some(bearer) = bearer {
            let api_token = bearer.0;
            usecases::authorize_organization_by_api_token(&*db, &api_token)?;
        } else {
            return Err(Error::Parameter(ParameterError::Unauthorized).into());
        }

        db.get_place_history(&id, None)?
    };
    Ok(Json(place_history.into()))
}

#[post("/places/<ids>/review", data = "<review>")]
pub fn post_places_review(
    account: Account,
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
        usecases::authorize_user_by_email(&*db, account.email(), Role::Scout)?.email
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
pub fn get_duplicates(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    ids: String,
) -> Result<Vec<(String, String, DuplicateType)>> {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let places = connections.shared()?.get_places(&ids)?;
    let results = usecases::find_duplicates(&search_engine, &places)?;
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
fn post_login(
    db: sqlite::Connections,
    mut cookies: Cookies,
    login: Json<json::Credentials>,
    jwt_state: State<jwt::JwtState>,
) -> Result<Option<ofdb_boundary::JwtToken>> {
    let login = usecases::Login::from(login.into_inner());
    {
        let credentials = usecases::Credentials {
            email: &login.email,
            password: &login.password,
        };
        usecases::login_with_email(&*db.shared()?, &credentials)?;
    }

    let mut response = None;
    if cfg!(feature = "jwt") {
        let token = jwt_state.generate_token(&login.email)?;
        response = Some(ofdb_boundary::JwtToken { token });
    }
    if cfg!(feature = "cookies") {
        cookies.add_private(
            Cookie::build(COOKIE_EMAIL_KEY, login.email)
                .same_site(rocket::http::SameSite::None)
                .finish(),
        );
    }
    Ok(Json(response))
}

#[post("/logout", format = "application/json")]
fn post_logout(
    mut cookies: Cookies,
    jwt_state: State<jwt::JwtState>,
    bearer: Option<Bearer>,
) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
    if cfg!(feature = "jwt") {
        if let Some(bearer) = bearer {
            jwt_state.blacklist_token(bearer.0);
        }
    }
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
    account: Account,
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
    let email = account.email();
    usecases::subscribe_to_bbox(&*db.exclusive()?, email.to_string(), bbox)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(db: sqlite::Connections, account: Account) -> Result<()> {
    let email = account.email();
    usecases::unsubscribe_all_bboxes(&*db.exclusive()?, &email)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(
    db: sqlite::Connections,
    account: Account,
) -> Result<Vec<json::BboxSubscription>> {
    let email = account.email();
    let user_subscriptions = usecases::get_bbox_subscriptions(&*db.shared()?, &email)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.id.into(),
            south_west_lat: s.bbox.southwest().lat().to_deg(),
            south_west_lng: s.bbox.southwest().lng().to_deg(),
            north_east_lat: s.bbox.northeast().lat().to_deg(),
            north_east_lng: s.bbox.northeast().lng().to_deg(),
        })
        .collect();
    Ok(Json(user_subscriptions))
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
fn entries_csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    bearer: Option<Bearer>,
    account: Account,
    query: Form<search::SearchQuery>,
) -> result::Result<Content<String>, AppError> {
    let db = connections.shared()?;

    let moderated_tags = if let Some(bearer) = bearer {
        let api_token = bearer.0;
        let org = usecases::authorize_organization_by_api_token(&*db, &api_token)?;
        org.moderated_tags
    } else {
        vec![]
    };

    let user = usecases::authorize_user_by_email(&*db, account.email(), Role::Scout)?;

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
        usecases::search(&*db, &search_engine, req, limit)?
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
                    let place = usecases::export_place(
                        place,
                        user.role,
                        moderated_tags
                            .iter()
                            .map(|moderated_tag| moderated_tag.label.as_str()),
                    );
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

    let buf: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buf);

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
                        ParameterError::Forbidden | ParameterError::ModeratedTag => {
                            Status::Forbidden
                        }
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
