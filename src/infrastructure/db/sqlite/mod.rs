mod connection;
mod models;
mod schema;
mod util;

use anyhow::Result as Fallible;
use diesel::{r2d2, sqlite::SqliteConnection};
use owning_ref::{RwLockReadGuardRef, RwLockWriteGuardRefMut};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

pub struct Connection(SqliteConnection);

impl diesel::connection::SimpleConnection for Connection {
    fn batch_execute(&self, x: &str) -> std::result::Result<(), diesel::result::Error> {
        self.0.batch_execute(x)
    }
}

impl diesel::connection::Connection for Connection {
    type Backend = <SqliteConnection as diesel::connection::Connection>::Backend;
    type TransactionManager =
        <SqliteConnection as diesel::connection::Connection>::TransactionManager;
    fn establish(url: &str) -> std::result::Result<Self, diesel::ConnectionError> {
        Ok(Self(SqliteConnection::establish(url)?))
    }
    fn execute(&self, query: &str) -> std::result::Result<usize, diesel::result::Error> {
        self.0.execute(query)
    }
    fn query_by_index<T, U>(&self, source: T) -> diesel::QueryResult<Vec<U>>
    where
        T: diesel::query_builder::AsQuery,
        T::Query:
            diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
        Self::Backend: diesel::types::HasSqlType<T::SqlType>,
        U: diesel::Queryable<T::SqlType, Self::Backend>,
    {
        self.0.query_by_index(source)
    }
    fn query_by_name<T, U>(&self, source: &T) -> diesel::QueryResult<Vec<U>>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
        U: diesel::query_source::QueryableByName<Self::Backend>,
    {
        self.0.query_by_name(source)
    }
    fn execute_returning_count<T>(&self, source: &T) -> diesel::QueryResult<usize>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
    {
        self.0.execute_returning_count(source)
    }
    fn transaction_manager(&self) -> &<Self as diesel::Connection>::TransactionManager {
        self.0.transaction_manager()
    }
}

pub type ConnectionManager = r2d2::ConnectionManager<Connection>;
pub type ConnectionPool = r2d2::Pool<ConnectionManager>;
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

pub type SharedConnectionPool = Arc<RwLock<ConnectionPool>>;

pub struct DbReadOnly<'a> {
    _locked_pool: RwLockReadGuardRef<'a, ConnectionPool>,
    conn: PooledConnection,
}

impl<'a> DbReadOnly<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = RwLockReadGuardRef::new(pool.read().unwrap_or_else(|err| {
            error!("Failed to lock database connection pool for read-only access");
            err.into_inner()
        }));
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
    _locked_pool: RwLockWriteGuardRefMut<'a, ConnectionPool>,
    conn: PooledConnection,
}

impl<'a> DbReadWrite<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = RwLockWriteGuardRefMut::new(pool.write().unwrap_or_else(|err| {
            error!("Failed to lock database connection pool for read/write access");
            err.into_inner()
        }));
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
