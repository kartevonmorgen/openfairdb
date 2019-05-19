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
mod register;
mod view;

const MAP_JS: &str = include_str!("map.js");
const MAIN_CSS: &str = include_str!("main.css");

type Result<T> = std::result::Result<T, AppError>;

fn check_role(db: &dyn Db, account: &Account, role: Role) -> Result<User> {
    if let Some(user) = db.get_users_by_email(account.email())?.first() {
        if user.role == role {
            return Ok(user.to_owned());
        }
    }
    return Err(Error::Parameter(ParameterError::Unauthorized).into());
}

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
        let admin = check_role(&*db, &account, Role::Admin)?;
        let users = db.get_users_by_email(&email)?;
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
    let (user, e, ratings): (Option<User>, _, _) = {
        let db = pool.shared()?;
        let e = db.get_entry(id.as_str())?;
        let ratings = db.load_ratings_of_entry(&e.id)?;
        let ratings_with_comments = db.zip_ratings_with_comments(ratings)?;
        let user = if let Some(a) = account {
            db.get_users_by_email(a.email())?
                .first()
                .map(|u| u.to_owned())
        } else {
            None
        };
        (user, e, ratings_with_comments)
    };
    Ok(match user {
        Some(u) => view::entry(Some(&u.email), (e, ratings, u.role).into()),
        None => view::entry(None, (e, ratings).into()),
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
    let entry_count = db.count_entries()?;
    let user_count = db.count_users()?;
    let event_count = db.count_events()?;
    let users = db.get_users_by_email(account.email())?;
    if let Some(user) = users.first().cloned() {
        if user.role == Role::Admin {
            return Ok(view::dashboard(view::DashBoardPresenter {
                user,
                entry_count,
                event_count,
                tag_count,
                user_count,
            }));
        }
    }
    Err(Error::Parameter(ParameterError::Unauthorized).into())
}

#[derive(FromForm)]
pub struct ArchiveAction {
    ids: String,
    entry_id: String,
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
            Redirect::to(uri!(get_entry:d.entry_id)),
            "Failed to achive the comment.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.entry_id))),
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
            Redirect::to(uri!(get_entry:d.entry_id)),
            "Failed to archive the rating.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_entry:d.entry_id))),
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
        register::get_email_confirmation
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::tantivy;
    use crate::ports::web::tests::prelude::*;

    fn setup() -> (
        rocket::local::Client,
        sqlite::Connections,
        tantivy::SearchEngine,
    ) {
        crate::ports::web::tests::setup(vec![("/", super::routes())])
    }

    mod events {
        use super::*;

        #[test]
        fn get_a_list_of_all_events() {
            let (client, db, _) = setup();
            let events = vec![
                Event {
                    id: "1234".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::hours(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
                Event {
                    id: "5678".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
                Event {
                    id: "0000".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::days(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
            ];

            {
                let db_conn = db.exclusive().unwrap();
                for e in events {
                    db_conn.create_event(e).unwrap();
                }
            }

            let mut res = client.get("/events").dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("<li><a href=\"/events/1234\">"));
            assert!(body_str.contains("<li><a href=\"/events/5678\">"));
            assert!(!body_str.contains("<li><a href=\"/events/0000\">"));
        }

        #[test]
        fn get_a_list_of_events_filtered_by_tags() {
            let (client, db, _) = setup();
            let events = vec![
                Event {
                    id: "1234".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::hours(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
                Event {
                    id: "5678".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bli".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
                Event {
                    id: "0000".into(),
                    title: "x".into(),
                    description: None,
                    start: chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::days(2))
                        .unwrap()
                        .naive_utc(),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["blub".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                    archived: None,
                },
            ];

            {
                let db_conn = db.exclusive().unwrap();
                for e in events {
                    db_conn.create_event(e).unwrap();
                }
            }

            let mut res = client.get("/events?tag=blub&tag=bli").dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("<li><a href=\"/events/1234\">"));
            assert!(body_str.contains("<li><a href=\"/events/5678\">"));
            // '0000' has "blub" but its too old
            assert!(!body_str.contains("<li><a href=\"/events/0000\">"));
        }

        #[test]
        fn get_a_single_event() {
            let (client, db, _) = setup();
            let events = vec![Event {
                id: "1234".into(),
                title: "A great event".into(),
                description: Some("Foo bar baz".into()),
                start: NaiveDateTime::from_timestamp(0, 0),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            }];

            {
                let db_conn = db.exclusive().unwrap();
                for e in events {
                    db_conn.create_event(e).unwrap();
                }
            }

            let mut res = client.get("/events/1234").dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("<h2>A great event</h2>"));
            assert!(body_str.contains("Foo bar baz</p>"));
        }

    }

    mod index {
        use super::*;
        #[test]
        fn get_the_index_html() {
            let (client, _db, _) = setup();
            let mut index = client.get("/").dispatch();
            assert_eq!(index.status(), Status::Ok);

            let mut index_html = client.get("/index.html").dispatch();
            assert_eq!(index_html.status(), Status::Ok);

            let index_str = index.body().and_then(|b| b.into_string()).unwrap();
            let index_html_str = index_html.body().and_then(|b| b.into_string()).unwrap();

            assert_eq!(index_html_str, index_str);
            assert!(index_str.contains("<form action=\"search\""));
            assert!(index_str.contains("<input type=\"text\""));
        }
    }

    mod entry {
        use super::*;
        use crate::core::usecases;
        use crate::infrastructure::flows;

        fn create_entry_with_rating(
            db: &sqlite::Connections,
            search: &mut tantivy::SearchEngine,
        ) -> (String, String, String) {
            let e = usecases::NewEntry {
                title: "entry".into(),
                description: "desc".into(),
                lat: 3.7,
                lng: -50.0,
                street: None,
                zip: None,
                city: None,
                country: None,
                email: None,
                telephone: None,
                homepage: None,
                categories: vec![],
                tags: vec![],
                license: "CC0-1.0".into(),
                image_url: None,
                image_link_url: None,
            };
            let e_id = flows::prelude::create_entry(db, search, e).unwrap();
            let r = usecases::RateEntry {
                title: "A rating".into(),
                comment: "Foo".into(),
                context: RatingContext::Diversity,
                source: None,
                user: None,
                value: 1.into(),
                entry: e_id.clone(),
            };
            let (r_id, c_id) = flows::prelude::create_rating(db, search, r).unwrap();
            (e_id, r_id, c_id)
        }

        #[test]
        fn get_entry_details() {
            let (client, db, mut search) = setup();
            let (id, _, _) = create_entry_with_rating(&db, &mut search);
            let mut res = client.get(format!("/entries/{}", id)).dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(body_str.contains("<form"), false);
            assert_eq!(
                body_str.contains("action=\"/comments/actions/archive\""),
                false
            );
        }

        #[test]
        fn get_entry_details_as_admin() {
            let (client, db, mut search) = setup();
            let (id, _, _) = create_entry_with_rating(&db, &mut search);
            super::super::login::tests::register_user(&db, "foo@bar.com", "bazbaz", true);
            let mut user = db
                .shared()
                .unwrap()
                .get_users_by_email("foo@bar.com")
                .unwrap()
                .into_iter()
                .next()
                .unwrap();
            user.role = Role::Admin;
            db.exclusive().unwrap().update_user(&user).unwrap();
            let login_res = client
                .post("/login")
                .header(ContentType::Form)
                .body("email=foo%40bar.com&password=bazbaz")
                .dispatch();
            let mut res = client.get(format!("/entries/{}", id)).dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(body_str.contains("<form"), true);
            assert_eq!(
                body_str.contains("action=\"/comments/actions/archive\""),
                true
            );
        }

        #[test]
        fn archive_comment_as_guest() {
            let (client, db, mut search) = setup();
            let (e_id, r_id, c_id) = create_entry_with_rating(&db, &mut search);
            let res = client
                .post("/comments/actions/archive")
                .header(ContentType::Form)
                .body(format!("ids={}&entry_id={}", c_id, e_id))
                .dispatch();
            assert_eq!(res.status(), Status::NotFound);
        }

        #[test]
        fn archive_rating_as_guest() {
            let (client, db, mut search) = setup();
            let (e_id, r_id, c_id) = create_entry_with_rating(&db, &mut search);
            let res = client
                .post("/ratings/actions/archive")
                .header(ContentType::Form)
                .body(format!("ids={}&entry_id={}", r_id, e_id))
                .dispatch();
            assert_eq!(res.status(), Status::NotFound);
        }
    }

}
