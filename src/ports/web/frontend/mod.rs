use super::sqlite::DbConn;
use crate::core::{prelude::*, usecases};
use maud::Markup;
use rocket::{self, response::content::JavaScript, Route};

mod view;

const EVENT_JS: &str = include_str!("event.js");

#[get("/event.js")]
pub fn get_event_js() -> JavaScript<&'static str> {
    JavaScript(EVENT_JS)
}

#[get("/events/<id>")]
pub fn get_event(db: DbConn, id: String) -> Result<Markup> {
    let mut ev = usecases::get_event(&*db.read_only()?, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(view::event(ev))
}

#[get("/events")]
pub fn get_events(db: DbConn) -> Result<Markup> {
    let events = db.read_only()?.all_events()?;
    Ok(view::events(&events))
}

pub fn routes() -> Vec<Route> {
    routes![get_events, get_event, get_event_js]
}
