use anyhow::Result as Fallible;
use ofdb_db_sqlite::{Connections as ConnectionPool, DbReadOnly, DbReadWrite};
use rocket::{
    outcome::try_outcome,
    request::{FromRequest, Outcome},
    Request, State,
};
use std::ops::Deref;

// Wrapper to be able to implement `FromRequest`
#[derive(Clone)]
pub struct Connections(ConnectionPool);

impl Connections {
    pub fn shared(&self) -> Fallible<DbReadOnly> {
        self.0.shared()
    }

    pub fn exclusive(&self) -> Fallible<DbReadWrite> {
        self.0.exclusive()
    }
}

impl From<ConnectionPool> for Connections {
    fn from(conn: ConnectionPool) -> Self {
        Self(conn)
    }
}

impl Deref for Connections {
    type Target = ConnectionPool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Connections {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let connections = try_outcome!(request.guard::<&State<Connections>>().await);
        Outcome::Success(connections.inner().clone())
    }
}
