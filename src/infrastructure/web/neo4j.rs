use r2d2_cypher::CypherConnectionManager;
use r2d2::{Pool, Error};

static POOL_SIZE: u32 = 5;

pub type ConnectionPool = Pool<CypherConnectionManager>;

pub fn create_connection_pool(db_url: &str) -> Result<ConnectionPool, Error> {
    let manager = CypherConnectionManager { url: db_url.into() };
    Pool::builder()
        .max_size(POOL_SIZE)
        .build(manager)
}
