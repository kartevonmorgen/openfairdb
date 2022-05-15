use std::{
    io,
    ops::{Deref, DerefMut},
};

#[cfg(not(test))]
use diesel::r2d2::PoolError;
use diesel::r2d2::{ManageConnection, Pool, PooledConnection};
use rocket::{
    http::Status,
    request::{self, FromRequest},
    Outcome, Request, State,
};

use crate::core::usecases::tests::MockDb;

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl ManageConnection for MockDbConnectionManager {
    type Connection = MockDb;
    type Error = io::Error;

    fn connect(&self) -> Result<MockDb, io::Error> {
        Ok(MockDb::default())
    }

    fn is_valid(&self, _: &mut MockDb) -> Result<(), io::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut MockDb) -> bool {
        false
    }
}

pub type ConnectionManager = MockDbConnectionManager;
pub type ConnectionPool = Pool<ConnectionManager>;
pub struct Connections(pub PooledConnection<ConnectionManager>);

#[cfg(not(test))]
pub fn init_connections(_: &str) -> Result<ConnectionPool, PoolError> {
    let manager = MockDbConnectionManager {};
    Pool::builder().max_size(1).build(manager)
}

impl<'a, 'r> FromRequest<'a, 'r> for Connections {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connections, ()> {
        let pool = request.guard::<State<ConnectionPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Connections(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for Connections {
    type Target = MockDb;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
