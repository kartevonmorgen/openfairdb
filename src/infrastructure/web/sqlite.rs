use r2d2_diesel::ConnectionManager;
use diesel::sqlite::SqliteConnection;
use r2d2::{Pool, Error};

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    Pool::builder()
        .max_size(POOL_SIZE)
        .build(manager)
}
