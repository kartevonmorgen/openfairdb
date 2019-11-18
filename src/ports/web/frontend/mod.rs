use crate::{
    core::{
        error::{Error, ParameterError},
        prelude::*,
        usecases,
    },
    infrastructure::{db::sqlite, error::*, flows::prelude::*},
    ports::web::{api::events::EventQuery, guards::*, tantivy::SearchEngine},
};
use chrono::NaiveDateTime;
use maud::Markup;
use num_traits::FromPrimitive;
use rocket::{
    self,
    http::RawStr,
    request::Form,
    response::{
        content::{Css, JavaScript},
        Flash, Redirect,
    },
    Route,
};

mod login;
mod password;
mod register;
#[cfg(test)]
mod tests;
mod view;

const MAP_JS: &str = include_str!("map.js");
const MAIN_CSS: &str = include_str!("main.css");

type Result<T> = std::result::Result<T, AppError>;

#[get("/")]
pub fn get_index_user(account: Account) -> Markup {
    view::index(Some(&account.email()))
}

#[get("/", rank = 2)]
pub fn get_index() -> Markup {
    view::index(None)
}

#[get("/index.html")]
pub fn get_index_html() -> Markup {
    view::index(None)
}

#[get("/search?<q>&<limit>")]
pub fn get_search(search_engine: SearchEngine, q: &RawStr, limit: Option<usize>) -> Result<Markup> {
    let q = q.url_decode()?;
    let entries = usecases::global_search(&search_engine, &q, limit.unwrap_or(10))?;
    Ok(view::search_results(None, &q, &entries))
}

#[get("/search-users?<email>")]
pub fn get_search_users(
    pool: sqlite::Connections,
    email: &RawStr,
    account: Account,
) -> Result<Markup> {
    let email = email.url_decode()?;
    {
        let db = pool.shared()?;
        let admin = usecases::authorize_user_by_email(&*db, account.email(), Role::Admin)?;
        let users: Vec<_> = db.try_get_user_by_email(&email)?.into_iter().collect();
        Ok(view::user_search_result(&admin.email, &users))
    }
}

#[derive(FromForm)]
pub struct ChangeUserRoleAction {
    email: String,
    role: u8,
}

#[post("/change-user-role", data = "<data>")]
pub fn post_change_user_role(
    db: sqlite::Connections,
    account: Account,
    data: Form<ChangeUserRoleAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();
    match Role::from_u8(d.role) {
        None => Err(Flash::error(
            Redirect::to(uri!(get_search_users:d.email)),
            "Failed to change user role: invalid role.",
        )),
        Some(role) => match change_user_role(&db, account.email(), &d.email, role) {
            Err(_) => Err(Flash::error(
                Redirect::to(uri!(get_search_users:d.email)),
                "Failed to change user role.",
            )),
            Ok(_) => Ok(Redirect::to(uri!(get_search_users:d.email))),
        },
    }
}

#[get("/map.js")]
pub fn get_map_js() -> JavaScript<&'static str> {
    JavaScript(MAP_JS)
}

#[get("/main.css")]
pub fn get_main_css() -> Css<&'static str> {
    Css(MAIN_CSS)
}

#[get("/entries/<id>")]
pub fn get_entry(
    pool: sqlite::Connections,
    id: &RawStr,
    account: Option<Account>,
) -> Result<Markup> {
    //TODO: dry out
    let (user, place, ratings): (Option<User>, _, _) = {
        let db = pool.shared()?;
        let (place, _) = db.get_place(id.as_str())?;
        let ratings = db.load_ratings_of_place(place.uid.as_ref())?;
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
pub fn get_event(db: sqlite::Connections, id: &RawStr) -> Result<Markup> {
    let mut ev = usecases::get_event(&*db.shared()?, &id)?;
    // TODO:Make sure within usecase that the creator email
    // is not shown to unregistered users
    ev.created_by = None;
    Ok(view::event(None, ev))
}

#[get("/events?<query..>")]
pub fn get_events(db: sqlite::Connections, query: EventQuery) -> Result<Markup> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }

    let start_min = query
        .start_min
        .map(|x| NaiveDateTime::from_timestamp(x, 0))
        .unwrap_or_else(|| {
            chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::days(1))
                .unwrap()
                .naive_utc()
        });

    let events = usecases::query_events(
        &*db.shared()?,
        query.tags,
        query.bbox,
        Some(start_min),
        query.start_max.map(|x| NaiveDateTime::from_timestamp(x, 0)),
        query.created_by,
        None,
    )?;

    Ok(view::events(&events))
}

#[get("/dashboard")]
pub fn get_dashboard(db: sqlite::Connections, account: Account) -> Result<Markup> {
    let db = db.shared()?;
    let tag_count = db.count_tags()?;
    let entry_count = db.count_places()?;
    let user_count = db.count_users()?;
    let event_count = db.count_events()?;
    let user = db
        .try_get_user_by_email(account.email())?
        .ok_or(Error::Parameter(ParameterError::Unauthorized))?;
    if user.role == Role::Admin {
        return Ok(view::dashboard(view::DashBoardPresenter {
            user,
            entry_count,
            event_count,
            tag_count,
            user_count,
        }));
    }
    Err(Error::Parameter(ParameterError::Unauthorized).into())
}

#[derive(FromForm)]
pub struct ArchiveAction {
    ids: String,
    place_uid: String,
}

#[post("/comments/actions/archive", data = "<data>")]
pub fn post_comments_archive(
    account: Account,
    db: sqlite::Connections,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    //TODO: dry out
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_comments(&db, account.email(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry:d.place_uid)),
            "Failed to achive the comment.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_uid))),
    }
}

#[post("/ratings/actions/archive", data = "<data>")]
pub fn post_ratings_archive(
    account: Account,
    db: sqlite::Connections,
    mut search_engine: SearchEngine,
    data: Form<ArchiveAction>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let d = data.into_inner();
    let ids: Vec<_> = d.ids.split(',').filter(|id| !id.is_empty()).collect();
    match archive_ratings(&db, &mut search_engine, account.email(), &ids) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_entry:d.place_uid)),
            "Failed to archive the rating.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.place_uid))),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        get_index_user,
        get_index,
        get_index_html,
        get_dashboard,
        get_search,
        get_entry,
        get_events,
        get_event,
        get_main_css,
        get_map_js,
        get_search_users,
        post_comments_archive,
        post_ratings_archive,
        post_change_user_role,
        login::get_login,
        login::get_login_user,
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
