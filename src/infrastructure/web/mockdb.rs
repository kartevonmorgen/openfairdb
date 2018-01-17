use business::usecase::tests::MockDb;
use std::io;
use diesel::r2d2::{ManageConnection, Pool, PoolError, PooledConnection};
use std::ops::{Deref, DerefMut};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl ManageConnection for MockDbConnectionManager {
    type Connection = MockDb;
    type Error = io::Error;

    fn connect(&self) -> Result<MockDb, io::Error> {
        Ok(MockDb::new())
    }

    fn is_valid(&self, _: &mut MockDb) -> Result<(), io::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut MockDb) -> bool {
        false
    }
}

pub type ConnectionPool = Pool<MockDbConnectionManager>;

pub struct DbConn(pub PooledConnection<MockDbConnectionManager>);

pub fn create_connection_pool(_: &str) -> Result<ConnectionPool, PoolError> {
    let manager = MockDbConnectionManager {};
    Pool::builder().max_size(1).build(manager)
}

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<ConnectionPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = MockDb;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
