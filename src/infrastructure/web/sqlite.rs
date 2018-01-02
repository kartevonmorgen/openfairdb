use r2d2_diesel::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use r2d2::Pool;
use super::super::error::AppError;

embed_migrations!();

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, AppError> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().max_size(POOL_SIZE).build(manager)?;

    embedded_migrations::run(&*pool.get()?)?;

    Ok(pool)
}
