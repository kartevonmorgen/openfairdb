use std::{collections::HashSet, env};

const DEFAULT_ACCEPTED_LICENSES: &str = "CC0-1.0,ODbL-1.0";
const DEFAULT_DB_URL: &str = "openfair.db";
const DB_CONNECTION_POOL_SIZE: u32 = 10;
const DEFAULT_PROTECT_WITH_CAPTCHA: bool = false;

#[derive(Debug, Clone)]
pub struct Cfg {
    pub accepted_licenses: HashSet<String>,
    pub db_url: String,
    pub db_connection_pool_size: u32,
    pub protect_with_captcha: bool,
    pub opencage_api_key: Option<String>,
}

impl Cfg {
    pub fn from_env_or_default() -> Self {
        let mut cfg = Self::default();
        if let Ok(l) = env::var("ACCEPTED_LICENSES") {
            cfg.accepted_licenses = l.split(',').map(ToString::to_string).collect();
        }
        if let Ok(db_url) = env::var("DATABASE_URL") {
            cfg.db_url = db_url;
        }
        if let Ok(p) = env::var("PROTECT_WITH_CAPTCHA").map(|s| s.to_lowercase()) {
            cfg.protect_with_captcha = p == "true" || p == "1" || p == "yes";
        }
        match env::var("OPENCAGE_API_KEY") {
            Ok(key) => {
                cfg.opencage_api_key = Some(key);
            }
            Err(_) => {
                log::warn!("No OpenCage API key found");
            }
        };
        cfg
    }
}

impl Default for Cfg {
    fn default() -> Self {
        let accepted_licenses = DEFAULT_ACCEPTED_LICENSES
            .split(',')
            .map(ToString::to_string)
            .collect();
        let db_url = DEFAULT_DB_URL.to_string();
        let db_connection_pool_size = DB_CONNECTION_POOL_SIZE;
        let protect_with_captcha = DEFAULT_PROTECT_WITH_CAPTCHA;
        let opencage_api_key = None;
        Self {
            accepted_licenses,
            db_url,
            db_connection_pool_size,
            protect_with_captcha,
            opencage_api_key,
        }
    }
}
