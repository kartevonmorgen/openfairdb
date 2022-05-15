mod connection;
mod models;
mod schema;
mod util;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::Result as Fallible;
use diesel::{r2d2, sqlite::SqliteConnection};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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
            error!("Failed to obtain pooled database connection for read-only access");
            err
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
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

impl<'a> DbReadWrite<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = pool.write();
        let conn = locked_pool.get().map_err(|err| {
            error!("Failed to obtain pooled database connection for read/write access");
            err
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn,
        })
    }
}

impl<'a> Deref for DbReadWrite<'a> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<'a> DerefMut for DbReadWrite<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.conn
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
