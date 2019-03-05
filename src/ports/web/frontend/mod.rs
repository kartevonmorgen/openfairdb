use crate::core::{prelude::*, usecases};
use crate::infrastructure::db::sqlite;
use maud::Markup;
use rocket::{
    self,
    response::content::{Css, JavaScript},
    Route,
};

mod view;

const MAP_JS: &str = include_str!("map.js");
const MAIN_CSS: &str = include_str!("main.css");

use crate::ports::web::tantivy::SearchEngine;
use rocket::http::RawStr;

#[get("/")]
pub fn get_index() -> Markup {
    view::index()
}

#[get("/index.html")]
pub fn get_index_html() -> Markup {
    view::index()
}

#[get("/search?<q>&<limit>")]
pub fn get_search(search_engine: SearchEngine, q: &RawStr, limit: Option<usize>) -> Result<Markup> {
    let entries = usecases::global_search(&search_engine, q.as_str(), limit.unwrap_or(10))?;
    Ok(view::search_results(q.as_str(), &entries))
}

#[get("/map.js")]
pub fn get_map_js() -> JavaScript<&'static str> {
    JavaScript(MAP_JS)
}

#[get("/main.css")]
pub fn get_main_css() -> Css<&'static str> {
    Css(MAIN_CSS)
}

#[get("/events/<id>")]
pub fn get_event(db: sqlite::Connections, id: &RawStr) -> Result<Markup> {
    let mut ev = usecases::get_event(&*db.shared().map_err(RepoError::from)?, &id)?;
    // TODO:
    // Make sure within usecase that the creator email
    // is not shown to unregistered users
    ev.created_by = None;
    Ok(view::event(ev))
}

#[get("/entries/<id>")]
pub fn get_entry(db: sqlite::Connections, id: &RawStr) -> Result<Markup> {
    let e = db
        .shared()
        .map_err(RepoError::from)?
        .get_entry(id.as_str())?;
    Ok(view::entry(e))
}

#[get("/events")]
pub fn get_events(db: sqlite::Connections) -> Result<Markup> {
    let events = db.shared().map_err(RepoError::from)?.all_events()?;
    Ok(view::events(&events))
}

pub fn routes() -> Vec<Route> {
    routes![
        get_index,
        get_index_html,
        get_search,
        get_entry,
        get_events,
        get_event,
        get_main_css,
        get_map_js
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::web::tests::prelude::*;
    use chrono::*;

    fn setup() -> (rocket::local::Client, sqlite::Connections) {
        let (client, connections, _) =
            crate::ports::web::tests::setup(vec![("/", super::routes())]);
        (client, connections)
    }

    mod events {
        use super::*;

        #[test]
        fn get_a_list_of_all_events() {
            let (client, db) = setup();
            let events = vec![
                Event {
                    id: "1234".into(),
                    title: "x".into(),
                    description: None,
                    start: NaiveDateTime::from_timestamp(0, 0),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                },
                Event {
                    id: "5678".into(),
                    title: "x".into(),
                    description: None,
                    start: NaiveDateTime::from_timestamp(0, 0),
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec!["bla".into()],
                    homepage: None,
                    created_by: None,
                    registration: Some(RegistrationType::Email),
                    organizer: None,
                },
            ];

            {
                let mut db_conn = db.exclusive().unwrap();
                for e in events {
                    db_conn.create_event(e).unwrap();
                }
            }

            let mut res = client.get("/events").dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("<li><a href=\"/events/1234\">"));
            assert!(body_str.contains("<li><a href=\"/events/5678\">"));
        }

        #[test]
        fn get_a_single_event() {
            let (client, db) = setup();
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
            }];

            {
                let mut db_conn = db.exclusive().unwrap();
                for e in events {
                    db_conn.create_event(e).unwrap();
                }
            }

            let mut res = client.get("/events/1234").dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("<h2>A great event</h2>"));
            assert!(body_str.contains("<p>Foo bar baz</p>"));
        }

    }

    mod index {
        use super::*;
        #[test]
        fn get_the_index_html() {
            let (client, _db) = setup();
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

}
