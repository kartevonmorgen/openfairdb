use r2d2_cypher::CypherConnectionManager;
use r2d2::{self, Pool};
use rusted_cypher::GraphClient;
use std::{env,io};

static POOL_SIZE: u32 = 5;

lazy_static! {
    pub static ref DB_POOL: r2d2::Pool<CypherConnectionManager> = {
        let config = r2d2::Config::builder().pool_size(POOL_SIZE).build();
        let db_url = env::var(super::DB_URL_KEY).expect(&format!("{} must be set.", super::DB_URL_KEY));
        let manager = CypherConnectionManager { url: db_url.into() };
        Pool::new(config, manager).expect("Failed to create pool.")
    };
}

pub struct DB(r2d2::PooledConnection<CypherConnectionManager>);

impl DB {
    pub fn conn(&mut self) -> &mut GraphClient {
        &mut *self.0
    }
}

pub fn db() -> io::Result<DB> {
    match DB_POOL.get() {
        Ok(conn) => Ok(DB(conn)),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}
