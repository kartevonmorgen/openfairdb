#[macro_use]
extern crate diesel;

use anyhow::Result as Fallible;
use diesel::{r2d2, sqlite::SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use ofdb_core::{repositories as repo, usecases as uc};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    cell::{RefCell, RefMut},
    sync::Arc,
};

mod models;
mod repo_impl;
mod schema;
mod util;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

type Connection = SqliteConnection;

type ConnectionManager = r2d2::ConnectionManager<Connection>;
type ConnectionPool = r2d2::Pool<ConnectionManager>;
type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

type SharedConnectionPool = Arc<RwLock<ConnectionPool>>;

pub struct DbReadOnly<'a> {
    _locked_pool: RwLockReadGuard<'a, ConnectionPool>,
    conn: RefCell<PooledConnection>,
}

impl<'a> DbReadOnly<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = pool.read();
        let conn = locked_pool.get().inspect_err(|err| {
            log::error!("Failed to obtain pooled database connection for read-only access: {err}");
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn: RefCell::new(conn),
        })
    }
}

pub struct DbReadWrite<'a> {
    _locked_pool: RwLockWriteGuard<'a, ConnectionPool>,
    conn: RefCell<PooledConnection>,
}

pub struct DbConnection<'a> {
    conn: RefCell<&'a mut SqliteConnection>,
}

impl<'a> DbConnection<'a> {
    fn new(conn: &'a mut SqliteConnection) -> Self {
        Self {
            conn: RefCell::new(conn),
        }
    }
}

impl<'a> DbReadWrite<'a> {
    fn try_new(pool: &'a SharedConnectionPool) -> Fallible<Self> {
        let locked_pool = pool.write();
        let conn = locked_pool.get().inspect_err(|err| {
            log::error!("Failed to obtain pooled database connection for read/write access: {err}");
        })?;
        Ok(Self {
            _locked_pool: locked_pool,
            conn: RefCell::new(conn),
        })
    }

    pub fn transaction<T, F, E>(&mut self, f: F) -> Result<T, uc::Error>
    where
        F: FnOnce(&DbConnection) -> Result<T, E>,
        E: Into<uc::Error>,
    {
        let mut usecase_error = None;
        use diesel::Connection;
        // transaction closure. Otherwise it would be inaccessible, because
        // it cannot be borrowed twice. Consequently it has to be passed
        // along into the FnOnce() closure.
        self.conn
            .borrow_mut()
            .transaction(|conn| {
                // TODO: Diesel v2.0 requires a mutable borrow of the connection
                // and will pass this mutable borrow into the transaction as
                // an additional parameter.
                f(&DbConnection::new(conn))
                    .map_err(Into::into)
                    .map_err(|err| {
                        usecase_error = Some(err);
                        diesel::result::Error::RollbackTransaction
                    })
            })
            .map_err(|err| {
                if let Some(usecase_error) = usecase_error {
                    debug_assert!(matches!(err, diesel::result::Error::RollbackTransaction));
                    usecase_error
                } else {
                    let err = match err {
                        diesel::result::Error::NotFound => repo::Error::NotFound,
                        _ => repo::Error::Other(err.into()),
                    };
                    uc::Error::Repo(err)
                }
            })
    }

    fn sqlite_conn(&self) -> RefMut<PooledConnection> {
        self.conn.borrow_mut()
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

/// Configure the database engine
///
/// The implementation of the repositories and use cases relies on a proper
/// configuration of the database engine like the behavior, e.g. recursive
/// cascading deletes.
///
/// Some values like the text encoding can only be changed once after the
/// database has initially been created.
pub fn initialize_database(connection: &mut SqliteConnection) -> Fallible<()> {
    use diesel::RunQueryDsl as _;
    diesel::sql_query(r#"
PRAGMA journal_mode = WAL;        -- better write-concurrency
PRAGMA synchronous = NORMAL;      -- fsync only in critical moments, safe for journal_mode = WAL
PRAGMA wal_autocheckpoint = 1000; -- write WAL changes back every 1000 pages (default), for an in average 1MB WAL file
PRAGMA wal_checkpoint(TRUNCATE);  -- free some space by truncating possibly massive WAL files from the last run
PRAGMA secure_delete = 0;         -- avoid some disk I/O
PRAGMA automatic_index = 1;       -- detect and log missing indexes
PRAGMA foreign_keys = 1;          -- check foreign key constraints
PRAGMA defer_foreign_keys = 1;    -- delay enforcement of foreign key constraints until commit
PRAGMA recursive_triggers = 1;    -- for recursive ON CASCADE DELETE actions
PRAGMA encoding = 'UTF-8';
"#).execute(connection)?;
    Ok(())
}

impl Connections {
    pub fn init(url: &str, pool_size: u32) -> Fallible<Self> {
        // Establish a test connection before creating the connection pool to fail early.
        // If the given file is inaccessible r2d2 (Diesel 1.4.8) seems to do multiple retries
        // and logs errors instead of simply failing and returning and error immediately.
        // Malformed example file name for testing: ":/tmp/ofdb.sqlite"
        use diesel::Connection as _;
        let _ = diesel::SqliteConnection::establish(url)?;
        // The test connection is dropped immediately without using it
        // and missing files should have been created after reaching
        // this point.
        let manager = ConnectionManager::new(url);
        let pool = ConnectionPool::builder()
            .max_size(pool_size)
            .build(manager)?;
        initialize_database(&mut *pool.get()?)?;
        Ok(Self::new(pool))
    }

    fn new(pool: ConnectionPool) -> Self {
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

pub fn run_embedded_database_migrations(conn: DbReadWrite<'_>) {
    log::info!("Running embedded database migrations");
    conn.sqlite_conn()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
}
