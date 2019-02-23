use crate::infrastructure::error::AppError;
use crate::core::error::{RepoError, Error};
use diesel::r2d2::{ConnectionManager, Pool, self};
use diesel::sqlite::SqliteConnection;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

embed_migrations!();

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct DbConn {
    pool: ConnectionPool,
}

impl DbConn {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    pub fn pooled(&self) -> Result<PooledConnection, Error> {
        self.pool.get().map_err(|err| {
            error!("Failed to obtain pooled database connection");
            Error::Repo(RepoError::Other(Box::new(err)))
        }
    }
}

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
        Outcome::Success(DbConn::new(pool.inner().clone()))
    }
}
