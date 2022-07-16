use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};

pub use crate::infrastructure::db::tantivy::*;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SearchEngine {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let search_engine = try_outcome!(request.guard::<&State<SearchEngine>>().await);
        Outcome::Success(search_engine.inner().clone())
    }
}
