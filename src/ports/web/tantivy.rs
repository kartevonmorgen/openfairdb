pub use crate::infrastructure::db::tantivy::*;

use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

impl<'a, 'r> FromRequest<'a, 'r> for SearchEngine {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<SearchEngine, ()> {
        let search_engine = request.guard::<State<SearchEngine>>()?;
        Outcome::Success(search_engine.clone())
    }
}
