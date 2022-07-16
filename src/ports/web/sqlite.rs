use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};

pub use crate::infrastructure::db::sqlite::*;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Connections {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let connections = try_outcome!(request.guard::<&State<Connections>>().await);
        Outcome::Success(connections.inner().clone())
    }
}
