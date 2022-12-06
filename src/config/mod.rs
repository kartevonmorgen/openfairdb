use anyhow::{anyhow, Result};
use ofdb_entities::email::EmailAddress;
use std::{
    collections::HashSet,
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::Duration,
};

mod raw;

const DEFAULT_CONFIG_FILE_NAME: &str = "openfairdb.toml";

const ENV_NAME_DB_URL: &str = "DATABASE_URL";

pub struct Config {
    pub db: Db,
    pub entries: Entries,
    pub webserver: WebServer,
    pub email: Email,
    pub geocoding: Geocoding,
    pub reminders: Reminders,
}

impl Config {
    pub fn try_load_from_file_or_default<P: AsRef<Path>>(file_path: Option<P>) -> Result<Self> {
        let file_path: &Path = file_path.as_ref().map(|p| p.as_ref()).unwrap_or_else(|| {
            log::info!("No configuration file specified. load {DEFAULT_CONFIG_FILE_NAME}");
            Path::new(DEFAULT_CONFIG_FILE_NAME)
        });

        let raw_config = match fs::read_to_string(file_path) {
            Ok(cfg_string) => toml::from_str(&cfg_string)?,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    log::info!(
                        "{DEFAULT_CONFIG_FILE_NAME} not found => load default configuration."
                    );
                    Ok(raw::Config::default())
                }
                _ => Err(err),
            }?,
        };
        let mut cfg = Self::try_from(raw_config)?;
        if let Ok(db_url) = env::var(ENV_NAME_DB_URL) {
            cfg.db.conn_sqlite = db_url;
        }
        Ok(cfg)
    }
}

pub struct Db {
    /// SQLite connection
    pub conn_sqlite: String,
    pub conn_pool_size: u8,
    /// File system directory for the full-text search index.
    pub index_dir: Option<PathBuf>,
}

pub struct Geocoding {
    pub gateway: Option<GeocodingGateway>,
}

pub enum GeocodingGateway {
    OpenCage { api_key: String },
}

pub struct Entries {
    pub accepted_licenses: HashSet<String>,
}

pub struct WebServer {
    pub protect_with_captcha: bool,
    pub enable_cors: bool,
}

pub struct Email {
    pub gateway: Option<EmailGateway>,
}

#[derive(Clone)]
pub enum EmailGateway {
    MailGun {
        api_url: String, // TODO: use url::Url
        api_key: String,
        domain: String,
        sender_address: EmailAddress,
    },
    Sendmail {
        sender_address: EmailAddress,
    },
    /// For local testing purposes
    EmailToJsonFile {
        /// File system directory for writing emails into JSON files.
        dir: PathBuf,
    },
}

pub struct Reminders {
    pub task_interval_time: Duration,
    pub send_max: u32,
    pub scouts: ScoutReminders,
    pub owners: OwnerReminders,
}

pub struct ScoutReminders {
    pub not_updated_for: Duration,
}

pub struct OwnerReminders {
    pub not_updated_for: Duration,
}

impl TryFrom<raw::Config> for Config {
    type Error = anyhow::Error;
    fn try_from(from: raw::Config) -> Result<Self> {
        let raw::Config {
            db,
            geocoding,
            entries,
            webserver,
            email,
            gateway,
            reminders,
        } = from;

        let raw::Db {
            connection_sqlite,
            connection_pool_size,
            index_dir,
        } = db.unwrap_or_default();

        let db = Db {
            conn_sqlite: connection_sqlite,
            conn_pool_size: connection_pool_size,
            index_dir,
        };

        let email_gateway = match email.and_then(|m| m.gateway) {
            Some(gw_name) => {
                let toml_name = toml::to_string(&gw_name).unwrap();
                let gateway = gateway.clone().unwrap_or_default();

                let gw = match gw_name {
                    raw::EmailGateway::Mailgun => {
                        let raw::MailGun {
                            api_key,
                            api_url,
                            domain,
                            sender_address,
                        } = gateway.mailgun.ok_or_else(|| {
                            anyhow!("Missing '{toml_name}' gateway configuration")
                        })?;
                        let sender_address = sender_address.parse()?;
                        let api_url = api_url.unwrap_or_else(|| {
                            format!("https://api.eu.mailgun.net/v3/{}/messages", domain)
                        });
                        log::info!("Use Mailgun gateway");
                        EmailGateway::MailGun {
                            api_key,
                            api_url,
                            domain,
                            sender_address,
                        }
                    }
                    raw::EmailGateway::Sendmail => {
                        let raw::Sendmail { sender_address } =
                            gateway.sendmail.ok_or_else(|| {
                                anyhow!("Missing '{toml_name}' gateway configuration")
                            })?;
                        let sender_address = sender_address.parse()?;
                        EmailGateway::Sendmail { sender_address }
                    }
                    raw::EmailGateway::EmailToJsonFile => {
                        let raw::EmailToJsonFile { dir } =
                            gateway.email_to_json_file.ok_or_else(|| {
                                anyhow!("Missing '{toml_name}' gateway configuration")
                            })?;

                        log::info!("Use JSON file email gateway ({})", dir.display());
                        EmailGateway::EmailToJsonFile { dir }
                    }
                };
                Some(gw)
            }
            None => None,
        };

        let email = Email {
            gateway: email_gateway,
        };

        let raw::Entries { accepted_licenses } = entries.unwrap_or_default();

        if accepted_licenses.is_empty() {
            return Err(anyhow!("No accepted licences defined"));
        }
        let entries = Entries { accepted_licenses };

        let geo_gateway = match geocoding.and_then(|g| g.gateway) {
            Some(gw_name) => {
                let toml_name = toml::to_string(&gw_name).unwrap();
                let gateway = gateway.ok_or_else(|| anyhow!("Missing gateway configuration"))?;
                let gw = match gw_name {
                    raw::GeocodingGateway::Opencage => {
                        let raw::OpenCage { api_key } = gateway.opencage.ok_or_else(|| {
                            anyhow!("Missing '{toml_name}' gateway configuration")
                        })?;
                        GeocodingGateway::OpenCage { api_key }
                    }
                };
                Some(gw)
            }
            None => None,
        };
        let geocoding = Geocoding {
            gateway: geo_gateway,
        };

        let raw::WebServer { captcha, cors } = webserver.unwrap_or_default();

        let webserver = WebServer {
            protect_with_captcha: captcha,
            enable_cors: cors,
        };

        let raw::Reminders {
            task_interval_time,
            send_max,
            scouts,
            owners,
        } = reminders.unwrap_or_default();

        let task_interval_time = task_interval_time.expect("Reminder task interval");
        let send_max = send_max.expect("Send max. reminders configuration");

        let raw::ScoutReminders { not_updated_for } = scouts.unwrap_or_default();
        let scouts = ScoutReminders { not_updated_for };

        let raw::OwnerReminders { not_updated_for } = owners.unwrap_or_default();
        let owners = OwnerReminders { not_updated_for };

        let reminders = Reminders {
            task_interval_time,
            send_max,
            scouts,
            owners,
        };

        Ok(Self {
            db,
            email,
            entries,
            geocoding,
            webserver,
            reminders,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_config() {
        let file: Option<&Path> = None;
        let _: Config = Config::try_load_from_file_or_default(file).unwrap();
    }
}
