use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use super::super::error::AppError;
use std::ops::{Deref, DerefMut};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

embed_migrations!();

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

pub struct DbConn(pub PooledConnection<ConnectionManager<SqliteConnection>>);

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, AppError> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().max_size(POOL_SIZE).build(manager)?;

    embedded_migrations::run(&*pool.get()?)?;

    Ok(pool)
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
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
