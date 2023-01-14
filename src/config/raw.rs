use duration_str::{deserialize_duration, deserialize_option_duration};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf, time::Duration};

const DEFAULT_CONFIG_FILE: &[u8] = include_bytes!("openfairdb.default.toml");

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub db: Option<Db>,
    pub geocoding: Option<Geocoding>,
    pub entries: Option<Entries>,
    pub webserver: Option<WebServer>,
    pub email: Option<Email>,
    pub gateway: Option<Gateway>,
    pub reminders: Option<Reminders>,
}

impl Default for Config {
    fn default() -> Self {
        let cfg: Self = toml::from_slice(DEFAULT_CONFIG_FILE).expect("Default configuration");
        cfg
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Db {
    pub connection_sqlite: String,
    pub connection_pool_size: u8,
    pub index_dir: Option<PathBuf>,
}

impl Default for Db {
    fn default() -> Self {
        Config::default().db.expect("DB configuration")
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Geocoding {
    pub gateway: Option<GeocodingGateway>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeocodingGateway {
    Opencage,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OpenCage {
    pub api_key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Entries {
    pub accepted_licenses: HashSet<String>,
}

impl Default for Entries {
    fn default() -> Self {
        Config::default().entries.expect("Entries configuration")
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct WebServer {
    pub captcha: bool,
    pub cors: bool,
}

impl Default for WebServer {
    fn default() -> Self {
        Config::default()
            .webserver
            .expect("Webserver configuration")
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Email {
    pub gateway: Option<EmailGateway>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EmailGateway {
    Mailgun,
    Sendmail,
    EmailToJsonFile,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Gateway {
    pub mailgun: Option<MailGun>,
    pub sendmail: Option<Sendmail>,
    pub email_to_json_file: Option<EmailToJsonFile>,
    pub opencage: Option<OpenCage>,
}

impl Default for Gateway {
    fn default() -> Self {
        Config::default().gateway.expect("Gateway configuration")
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MailGun {
    pub api_key: String,
    pub domain: String,
    pub sender_address: String,
    pub api_base_url: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Sendmail {
    pub sender_address: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct EmailToJsonFile {
    pub dir: PathBuf,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Reminders {
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub task_interval_time: Option<Duration>,
    pub send_max: Option<u32>,
    pub send_to: Option<Vec<RecipientRole>>,
    pub scouts: Option<ScoutReminders>,
    pub owners: Option<OwnerReminders>,
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub token_expire_in: Option<Duration>,
}

impl Default for Reminders {
    fn default() -> Self {
        Config::default()
            .reminders
            .expect("Reminders configuration")
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecipientRole {
    Scouts,
    Owners,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ScoutReminders {
    #[serde(deserialize_with = "deserialize_duration")]
    pub not_updated_for: Duration,
}

impl Default for ScoutReminders {
    fn default() -> Self {
        Reminders::default()
            .scouts
            .expect("Scout reminders configuration")
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OwnerReminders {
    #[serde(deserialize_with = "deserialize_duration")]
    pub not_updated_for: Duration,
}

impl Default for OwnerReminders {
    fn default() -> Self {
        Reminders::default()
            .owners
            .expect("Owner reminders configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_default_config_from_file() {
        let cfg: Config = toml::from_slice(DEFAULT_CONFIG_FILE).unwrap();
        assert!(cfg.db.is_some());
        assert!(cfg.webserver.is_some());
        assert!(cfg.reminders.is_some());
    }

    #[test]
    fn default_reminders_config() {
        let cfg = Reminders::default();
        assert!(cfg.task_interval_time.is_some());
        assert!(cfg.send_max.is_some());
        assert!(cfg.send_to.is_none());
        assert!(cfg.scouts.is_some());
        assert!(cfg.owners.is_some());
        assert!(cfg.token_expire_in.is_some());
    }

    #[test]
    fn parse_full_config_example_from_file() {
        let cfg_string = fs::read_to_string("src/config/openfairdb.full-example.toml").unwrap();
        let _: Config = toml::from_str(&cfg_string).unwrap();
    }
}
