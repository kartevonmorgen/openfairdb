use r2d2_diesel::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use r2d2::{self, Pool, InitializationError};

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, InitializationError> {
    let config = r2d2::Config::builder().pool_size(POOL_SIZE).build();
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    Pool::new(config, manager)
}
