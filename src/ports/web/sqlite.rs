use crate::core::error::{Error, RepoError};
use crate::infrastructure::error::AppError;
use diesel::r2d2::{self, ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use owning_ref::{RwLockReadGuardRef, RwLockWriteGuardRefMut};
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

embed_migrations!();

static POOL_SIZE: u32 = 10;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct DbConn {
    // Only a single connection with write access will be
    // handed out at a time from the pool. Multiple read
    // connections can be accessed concurrently. This locking
    // pattern around the connection pool prevents SQLITE_LOCKED
    // ("database is locked") errors that are causing internal
    // server errors and failed requests.
    pool: Arc<RwLock<ConnectionPool>>,
}

pub struct DbReadOnly<'a> {
    _locked_pool: RwLockReadGuardRef<'a, ConnectionPool>,
    conn: PooledConnection,
}

impl<'a> DbReadOnly<'a> {
    fn try_new(pool: &'a Arc<RwLock<ConnectionPool>>) -> Result<Self, Error> {
        let locked_pool = RwLockReadGuardRef::new(pool.read().unwrap_or_else(|err| {
            error!("Failed to lock database connection pool for read-only access");
            err.into_inner()
        }));
        let conn = locked_pool.get().map_err(|err| {
            error!("Failed to obtain pooled database connection for read-only access");
            Error::Repo(RepoError::Other(Box::new(err)))
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
    }
}

impl<'a> Deref for DbReadOnly<'a> {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

pub struct DbReadWrite<'a> {
    _locked_pool: RwLockWriteGuardRefMut<'a, ConnectionPool>,
    conn: PooledConnection,
}

impl<'a> DbReadWrite<'a> {
    fn try_new(pool: &'a Arc<RwLock<ConnectionPool>>) -> Result<Self, Error> {
        let locked_pool = RwLockWriteGuardRefMut::new(pool.write().unwrap_or_else(|err| {
            error!("Failed to lock database connection pool for read/write access");
            err.into_inner()
        }));
        let conn = locked_pool.get().map_err(|err| {
            error!("Failed to obtain pooled database connection for read/write access");
            Error::Repo(RepoError::Other(Box::new(err)))
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
    }
}

impl<'a> Deref for DbReadWrite<'a> {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<'a> DerefMut for DbReadWrite<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.conn
    }
}

impl DbConn {
    pub fn new(pool: ConnectionPool) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
        }
    }

    pub fn read_only<'a>(&'a self) -> Result<DbReadOnly<'a>, Error> {
        DbReadOnly::try_new(&self.pool)
    }

    pub fn read_write<'a>(&'a self) -> Result<DbReadWrite<'a>, Error> {
        DbReadWrite::try_new(&self.pool)
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
