use rocket::{
    request::{self, FromRequest},
    Outcome, Request, State,
};

pub use crate::infrastructure::db::sqlite::*;

impl<'a, 'r> FromRequest<'a, 'r> for Connections {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connections, ()> {
        let connections = request.guard::<State<Connections>>()?;
        Outcome::Success(connections.inner().clone())
    }
}
