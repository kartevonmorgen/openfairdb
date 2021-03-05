use std::{collections::HashSet, env};

const DEFAULT_ACCEPTED_LICENSES: &str = "CC0-1.0,ODbL-1.0";
const DEFAULT_DB_URL: &str = "openfair.db";
const DB_CONNECTION_POOL_SIZE: u32 = 10;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub accepted_licenses: HashSet<String>,
    pub db_url: String,
    pub db_connection_pool_size: u32,
}

impl Cfg {
    pub fn from_env_or_default() -> Self {
        let mut cfg = Self::default();
        if let Ok(l) = env::var("ACCEPTED_LICENSES") {
            cfg.accepted_licenses = l.split(",").map(ToString::to_string).collect();
        }
        if let Ok(db_url) = env::var("DATABASE_URL") {
            cfg.db_url = db_url;
        }
        cfg
    }
}

impl Default for Cfg {
    fn default() -> Self {
        let accepted_licenses = DEFAULT_ACCEPTED_LICENSES
            .split(",")
            .map(ToString::to_string)
            .collect();
        let db_url = DEFAULT_DB_URL.to_string();
        let db_connection_pool_size = DB_CONNECTION_POOL_SIZE;
        Self {
            accepted_licenses,
            db_url,
            db_connection_pool_size,
        }
    }
}
