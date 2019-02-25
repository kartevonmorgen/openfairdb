pub use crate::infrastructure::db::sqlite::*;

use rocket::{
    request::{self, FromRequest},
    Outcome, Request, State,
};

impl<'a, 'r> FromRequest<'a, 'r> for Connections {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connections, ()> {
        let connections = request.guard::<State<Connections>>()?;
        Outcome::Success(connections.inner().clone())
    }
}
