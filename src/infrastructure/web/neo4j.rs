use r2d2_cypher::CypherConnectionManager;
use r2d2::{self, Pool, InitializationError};

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<CypherConnectionManager>;

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, InitializationError> {
    let config = r2d2::Config::builder().pool_size(POOL_SIZE).build();
    let manager = CypherConnectionManager { url: db_url.into() };
    Pool::new(config, manager)
}
