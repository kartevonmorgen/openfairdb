use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

use maud::Markup;
use num_traits::FromPrimitive;
use rocket::{
    self,
    form::Form,
    get,
    http::{ContentType, Status},
    post,
    response::{
        content::{RawCss, RawHtml, RawJavaScript},
        Flash, Redirect,
    },
    routes, uri, FromForm, Route,
};
use rust_embed::RustEmbed;

use crate::{
    core::{prelude::*, usecases},
    web::{
        api::{events::EventQuery, ApiError},
        guards::*,
        sqlite,
        tantivy::SearchEngine,
    },
};
use ofdb_application::prelude::*;
use ofdb_core::{repositories::Error as RepoError, usecases::Error as ParameterError};

mod login;
mod password;
mod register;
mod view;

#[cfg(test)]
mod tests;

const MAP_JS: &str = include_str!("map.js");
const MAIN_CSS: &str = include_str!("main.css");

#[derive(RustEmbed)]
#[folder = "../ofdb-app-clearance/dist/"]
struct ClearanceAsset;

type Result<T> = std::result::Result<T, ApiError>;

#[get("/")]
pub fn get_index_user(auth: Auth) -> Markup {
    view::index(auth.account_email().ok())
}

#[get("/", rank = 2)]
pub fn get_index() -> Markup {
    view::index(None)
}

#[get("/index.html")]
pub fn get_index_html() -> Markup {
    view::index(None)
}

#[get("/clearance")]
pub fn get_clearance_index() -> Option<RawHtml<Cow<'static, [u8]>>> {
    ClearanceAsset::get("index.html").map(|html| RawHtml(html.data))
}

#[get("/clearance/<file..>")]
pub fn get_clearance(file: PathBuf) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let filename = file.display().to_string();
    let asset = ClearanceAsset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);
    Some((content_type, asset.data))
}

#[get("/search?<q>&<limit>")]
pub fn get_search(search_engine: SearchEngine, q: &str, limit: Option<usize>) -> Result<Markup> {
    let entries = usecases::global_search(&*search_engine, q, limit.unwrap_or(10))?;
    Ok(view::search_results(None, q, &entries))
}

#[get("/search-users?<email>")]
pub fn get_search_users(pool: sqlite::Connections, email: &str, auth: Auth) -> Result<Markup> {
    let email = email.parse()?;
    {
        let db = pool.shared()?;
        let admin = auth.user_with_min_role(&db, Role::Admin)?;
        let users: Vec<_> = db.try_get_user_by_email(&email)?.into_iter().collect();
        Ok(view::user_search_result(&admin.email, &users))
    }
}

#[derive(FromForm)]
pub struct ChangeUserRoleAction<'r> {
    email: &'r str,
    role: u8,
}

#[allow(clippy::result_large_err)]
#[post("/change-user-role", data = "<data>")]
pub fn post_change_user_role(
    db: sqlite::Connections,
    account: Account,
    data: Form<ChangeUserRoleAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();

    let Ok(email) = d.email.parse::<EmailAddress>() else {
        return Err(Flash::error(
            Redirect::to(uri!(get_search_users(d.email))),
            "Invalid email address",
        ));
    };

    let Some(role) = Role::from_u8(d.role) else {
       return Err(Flash::error(
           Redirect::to(uri!(get_search_users(email.as_str()))),
           "Failed to change user role: invalid role.",
       ));
    };

    match change_user_role(&db, account.email(), &email, role) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_search_users(email.as_str()))),
            "Failed to change user role.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_search_users(email.as_str())))),
    }
}

#[get("/map.js")]
pub fn get_map_js() -> RawJavaScript<&'static str> {
    RawJavaScript(MAP_JS)
}

#[get("/main.css")]
pub fn get_main_css() -> RawCss<&'static str> {
    RawCss(MAIN_CSS)
}

#[get("/places/<id>/history")]
pub fn get_place_history(db: sqlite::Connections, id: &str, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    let user = db
        .try_get_user_by_email(account.email())?
        .ok_or(ParameterError::Unauthorized)?;
    let place_history = {
        // The history contains e-mail addresses of registered users
        // and is only permitted for scouts and admins!
        usecases::authorize_user_by_email(&db, account.email(), Role::Scout)?;

        db.get_place_history(id, None)?
    };
    Ok(view::place_history(&user, &place_history))
}

#[get("/places/<id>/review")]
pub fn get_place_review(db: sqlite::Connections, id: &str, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    // Only scouts and admins are entitled to review places
    let reviewer_email =
        usecases::authorize_user_by_email(&db, account.email(), Role::Scout)?.email;
    let (place, review_status) = db.get_place(id)?;
    Ok(view::place_review(&reviewer_email, &place, review_status))
}

#[derive(FromForm)]
pub struct Review<'r> {
    pub comment: &'r str,
    pub status: i16,
}

#[allow(clippy::result_large_err)]
#[post("/places/<id>/review", data = "<review>")]
pub fn post_place_review(
    db: sqlite::Connections,
    search_engine: SearchEngine,
    id: &str,
    review: Form<Review>,
    account: Account,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let Review { status, comment } = review.into_inner();
    review_place(
        &db,
        account.email(),
        status,
        comment.to_string(),
        id,
        search_engine,
    )
    .map(|_| Redirect::to(uri!(get_entry(id))))
    .map_err(|_| {
        Flash::error(
            Redirect::to(uri!(get_place_review(id))),
            "Failed to archive the place.",
        )
    })
}

// TODO: Move this into the separate web application
// that uses the official JSON API.
#[get("/places/review-with-token?<token>&<status>")]
pub fn get_place_review_with_token(
    connections: sqlite::Connections,
    token: String,
    status: String,
) -> Result<Markup> {
    let review_status: ReviewStatus = status.parse().map_err(|_| {
        ApiError::OtherWithStatus(anyhow::anyhow!("Invalid review status"), Status::BadRequest)
    })?;
    let review_nonce = ReviewNonce::decode_from_str(&token)?;
    let now = Timestamp::now();
    // TODO: return place, revision and status and display it to the user.
    ofdb_application::prelude::review_place_with_review_nonce(
        &connections,
        review_nonce,
        review_status,
        now,
    )?;
    Ok(view::review_place_with_token())
}

fn review_place(
    db: &sqlite::Connections,
    email: &EmailAddress,
    status: i16,
    comment: String,
    id: &str,
    mut search_engine: SearchEngine,
) -> Result<()> {
    let reviewer_email = {
        let db = db.shared()?;
        usecases::authorize_user_by_email(&db, email, Role::Scout)?.email
    };
    let status = ReviewStatus::try_from(status)
        .map_err(|_| ParameterError::RatingContext(status.to_string()))?;
    // TODO: Record context information
    let context = None;
    let review = usecases::Review {
        context,
        reviewer_email,
        status,
        comment: Some(comment),
    };
    let update_count = review_places(db, &mut *search_engine, &[id], review)?;
    if update_count == 0 {
        return Err(RepoError::NotFound.into());
    }
    Ok(())
}

#[get("/entries/<id>")]
pub fn get_entry(pool: sqlite::Connections, id: &str, account: Option<Account>) -> Result<Markup> {
    //TODO: dry out
    let (user, place, ratings): (Option<User>, _, _) = {
        let db = pool.shared()?;
        let (place, _) = db.get_place(id)?;
        let ratings = db.load_ratings_of_place(place.id.as_ref())?;
        let ratings_with_comments = db.zip_ratings_with_comments(ratings)?;
        let user = if let Some(a) = account {
            db.try_get_user_by_email(a.email())?
        } else {
            None
        };
        (user, place, ratings_with_comments)
    };
    Ok(match user {
        Some(u) => view::entry(Some(&u.email), (place, ratings, u.role).into()),
        None => view::entry(None, (place, ratings).into()),
    })
}

#[get("/events/<id>")]
pub fn get_event(pool: sqlite::Connections, id: &str, account: Option<Account>) -> Result<Markup> {
    let (user, mut ev): (Option<User>, _) = {
        let db = pool.shared()?;
        let ev = usecases::get_event(&db, id)?;
        let user = if let Some(a) = account {
            db.try_get_user_by_email(a.email())?
        } else {
            None
        };
        (user, ev)
    };

    // TODO:Make sure within usecase that the creator email
    // is not shown to unregistered users
    ev.created_by = None;

    Ok(view::event(user, ev))
}

#[allow(clippy::result_large_err)]
#[post("/events/<id>/archive")]
pub fn post_archive_event(
    account: Account,
    pool: sqlite::Connections,
    mut search_engine: SearchEngine,
    id: &str,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let archived_by_email = pool
        .shared()
        .and_then(|db| {
            // Only scouts and admins are entitled to review events
            let user = usecases::authorize_user_by_email(&db, account.email(), Role::Scout)?;
            Ok(user.email)
        })
        .map_err(|_| {
            Flash::error(
                Redirect::to(uri!(get_event(id))),
                "Failed to archive the event.",
            )
        })?;
    archive_events(&pool, &mut *search_engine, &[id], &archived_by_email)
        .map_err(|_| {
            Flash::error(
                Redirect::to(uri!(get_event(id))),
                "Failed to archive the event.",
            )
        })
        .map(|update_count| {
            if update_count != 1 {
                log::info!("Archived more than one event: {}", update_count);
            }
            Redirect::to("/events") //TODO: use uri! macro
        })
}

#[get("/events?<query..>")]
pub fn get_events_chronologically(
    db: sqlite::Connections,
    search_engine: SearchEngine,
    query: EventQuery,
    account: Option<Account>,
) -> Result<Markup> {
    let mut query = query.into_inner();
    if query.created_by.is_some() {
        return Err(ParameterError::Unauthorized.into());
    }

    if query.start_min.is_none() && query.start_max.is_none() {
        let start_min = Timestamp::now()
            .checked_sub(time::Duration::days(1))
            .unwrap();
        query.start_min = Some(start_min);
    }

    let events = usecases::query_events(&db.shared()?, &*search_engine, query)?;
    let email = account.as_ref().map(Account::email);
    Ok(view::events(email, &events))
}

#[get("/dashboard")]
pub fn get_dashboard(db: sqlite::Connections, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    let tag_count = db.count_tags()?;
    let place_count = db.count_places()?;
    let user_count = db.count_users()?;
    let event_count = db.count_events()?;
    let user = db
        .try_get_user_by_email(account.email())?
        .ok_or(ParameterError::Unauthorized)?;
    if user.role == Role::Admin {
        return Ok(view::dashboard(view::DashBoardPresenter {
            user,
            place_count,
            event_count,
            tag_count,
            user_count,
        }));
    }
    Err(ParameterError::Unauthorized.into())
}

#[derive(FromForm)]
pub struct ArchiveAction<'r> {
    ids: &'r str,
    place_id: &'r str,
}

#[allow(clippy::result_large_err)]
#[post("/comments/actions/archive", data = "<data>")]
pub fn post_comments_archive(
    account: Account,
    db: sqlite::Connections,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    //TODO: dry out
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_comments(&db, account.email().clone(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry(d.place_id))),
            "Failed to archive the comment.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry(d.place_id)))),
    }
}

#[allow(clippy::result_large_err)]
#[post("/ratings/actions/archive", data = "<data>")]
pub fn post_ratings_archive(
    account: Account,
    db: sqlite::Connections,
    mut search_engine: SearchEngine,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_ratings(&db, &mut *search_engine, account.email().clone(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry(d.place_id))),
            "Failed to archive the rating.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry(d.place_id)))),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        get_clearance,
        get_clearance_index,
        get_index_user,
        get_index,
        get_index_html,
        get_dashboard,
        get_search,
        get_entry,
        get_place_history,
        get_place_review,
        get_place_review_with_token,
        post_place_review,
        get_events_chronologically,
        get_event,
        get_main_css,
        get_map_js,
        get_search_users,
        post_comments_archive,
        post_ratings_archive,
        post_change_user_role,
        post_archive_event,
        login::get_login,
        login::post_login,
        login::post_logout,
        register::get_register,
        register::post_register,
        register::get_email_confirmation,
        password::get_reset_password,
        password::post_reset_password_request,
        password::post_reset_password,
    ]
}
