#[macro_use]
extern crate diesel;

use anyhow::Result as Fallible;
use diesel::{r2d2, sqlite::SqliteConnection};
use ofdb_core::{repositories as repo, usecases as uc};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{ops::Deref, sync::Arc};
use thiserror::Error;

mod models;
mod repo_impl;
mod repo_wrapper;
mod schema;
mod util;

pub use repo_impl::from_diesel_err;
pub use repo_wrapper::*;

pub type Connection = SqliteConnection;

pub type ConnectionManager = r2d2::ConnectionManager<Connection>;
pub type ConnectionPool = r2d2::Pool<ConnectionManager>;
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

pub type SharedConnectionPool = Arc<RwLock<ConnectionPool>>;

pub struct DbReadOnly<'a> {
    _locked_pool: RwLockReadGuard<'a, ConnectionPool>,
    conn: PooledConnection,
}

impl<'a> DbReadOnly<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = pool.read();
        let conn = locked_pool.get().map_err(|err| {
            log::error!("Failed to obtain pooled database connection for read-only access");
            err
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
    }
    fn inner(&self) -> repo_impl::Connection<'_> {
        repo_impl::Connection::new(&self.conn)
    }
}

impl<'a> Deref for DbReadOnly<'a> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

pub struct DbReadWrite<'a> {
    _locked_pool: RwLockWriteGuard<'a, ConnectionPool>,
    conn: PooledConnection,
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error(transparent)]
    Usecase(#[from] uc::Error),
    #[error("Rollback transaction")]
    RollbackTransaction,
}

impl<'a> DbReadWrite<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = pool.write();
        let conn = locked_pool.get().map_err(|err| {
            log::error!("Failed to obtain pooled database connection for read/write access");
            err
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
    }
    pub fn transaction<T, F, E>(&self, f: F) -> Result<T, uc::Error>
    where
        F: FnOnce() -> Result<T, E>,
        E: Into<TransactionError>,
    {
        let mut usecase_error = None;
        use diesel::Connection;
        (&*self.conn)
            .transaction(|| {
                f().map_err(Into::into).map_err(|err| match err {
                    TransactionError::Usecase(err) => {
                        usecase_error = Some(err);
                        diesel::result::Error::RollbackTransaction
                    }
                    TransactionError::RollbackTransaction => {
                        diesel::result::Error::RollbackTransaction
                    }
                })
            })
            .map_err(|err| {
                if let Some(uc_err) = usecase_error {
                    uc_err
                } else {
                    uc::Error::Repo(match err {
                        diesel::result::Error::NotFound => repo::Error::NotFound,
                        _ => repo::Error::Other(err.into()),
                    })
                }
            })
    }

    pub fn inner(&self) -> repo_impl::Connection<'_> {
        repo_impl::Connection::new(&self.conn)
    }
    pub fn sqlite_conn(&self) -> &Connection {
        &self.conn
    }
}

#[derive(Clone)]
pub struct Connections {
    // Only a single connection with write access will be
    // handed out at a time from the pool. Multiple read
    // connections can be accessed concurrently. This locking
    // pattern around the connection pool prevents SQLITE_LOCKED
    // ("database is locked") errors that are causing internal
    // server errors and failed requests.
    pool: SharedConnectionPool,
}

impl Connections {
    pub fn init(url: &str, pool_size: u32) -> Fallible<Self> {
        let manager = ConnectionManager::new(url);
        let pool = ConnectionPool::builder()
            .max_size(pool_size)
            .build(manager)?;
        Ok(Self::new(pool))
    }

    pub fn new(pool: ConnectionPool) -> Self {
        Self {
            pool: Arc::new(RwLock::new(pool)),
        }
    }

    pub fn shared(&self) -> Fallible<DbReadOnly> {
        DbReadOnly::try_new(&self.pool)
    }

    pub fn exclusive(&self) -> Fallible<DbReadWrite> {
        DbReadWrite::try_new(&self.pool)
    }
}
