use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};
use std::ops::{Deref, DerefMut};

// Wrapper to be able to implement `FromRequest`
#[derive(Clone)]
pub struct SearchEngine(pub ofdb_db_tantivy::SearchEngine);

impl SearchEngine {
    #[cfg(test)]
    pub fn init_in_ram() -> Result<Self, anyhow::Error> {
        ofdb_db_tantivy::SearchEngine::init_in_ram().map(Self)
    }
}

impl Deref for SearchEngine {
    type Target = ofdb_db_tantivy::SearchEngine;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SearchEngine {
    fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SearchEngine {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let search_engine = try_outcome!(request.guard::<&State<SearchEngine>>().await);
        Outcome::Success(search_engine.inner().clone())
    }
}
