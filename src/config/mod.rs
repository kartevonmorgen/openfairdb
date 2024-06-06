use std::{
    collections::HashSet,
    fmt, fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::anyhow;
use thiserror::Error;

use ofdb_core::usecases::RecipientRole;
use ofdb_entities::email::EmailAddress;

mod raw;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Config {
    pub db: Db,
    pub entries: Entries,
    pub webserver: WebServer,
    pub email: Email,
    pub geocoding: Geocoding,
    pub reminders: Reminders,
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Config file not found")]
    NotFound,

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Config {
    pub fn try_load_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, LoadError> {
        let raw_config = try_load_raw_config_from_file(file_path)?;
        let cfg = Self::try_from(raw_config)?;
        Ok(cfg)
    }

    pub fn try_load_from_file_or_default<P: AsRef<Path>>(file_path: P) -> anyhow::Result<Self> {
        match Self::try_load_from_file(file_path.as_ref()) {
            Ok(cfg) => Ok(cfg),
            Err(err) => match err {
                LoadError::NotFound => {
                    log::info!(
                        "Configuration file {} not found: load default configuration.",
                        file_path.as_ref().display()
                    );
                    Ok(Self::default())
                }
                _ => Err(err.into()),
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::try_from(raw::Config::default()).expect("default config")
    }
}

fn try_load_raw_config_from_file<P: AsRef<Path>>(file_path: P) -> Result<raw::Config, LoadError> {
    let cfg_string = fs::read_to_string(file_path).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => LoadError::NotFound,
        _ => LoadError::Other(err.into()),
    })?;
    let raw_config = toml::from_str(&cfg_string)?;
    Ok(raw_config)
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Db {
    /// SQLite connection
    pub conn_sqlite: String,
    pub conn_pool_size: u8,
    /// File system directory for the full-text search index.
    pub index_dir: Option<PathBuf>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Geocoding {
    pub gateway: Option<GeocodingGateway>,
}

#[cfg_attr(test, derive(PartialEq))]
pub enum GeocodingGateway {
    OpenCage { api_key: String },
}

impl fmt::Debug for GeocodingGateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GeocodingGateway::OpenCage { api_key: _ } => {
                f.debug_struct("OpenCage").field("api_key", &"***").finish()
            }
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Entries {
    pub accepted_licenses: HashSet<String>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct WebServer {
    pub protect_with_captcha: bool,
    pub enable_cors: bool,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Email {
    pub gateway: Option<EmailGateway>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum EmailGateway {
    MailGun {
        api_base_url: String, // TODO: use url::Url
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

impl fmt::Debug for EmailGateway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailGateway::MailGun {
                api_base_url,
                api_key: _,
                domain,
                sender_address,
            } => f
                .debug_struct("MailGun")
                .field("api_base_url", &api_base_url)
                .field("api_key", &"***")
                .field("domain", &domain)
                .field("sender_address", &sender_address)
                .finish(),

            EmailGateway::Sendmail { sender_address } => f
                .debug_struct("Sendmail")
                .field("sender_address", &sender_address)
                .finish(),

            EmailGateway::EmailToJsonFile { dir } => f
                .debug_struct("EmailToJsonFile")
                .field("dir", &dir)
                .finish(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Reminders {
    pub task_interval_time: Duration,
    pub send_max: u32,
    pub scouts: ScoutReminders,
    pub owners: OwnerReminders,
    pub send_to: Vec<RecipientRole>,
    pub send_bcc: Vec<EmailAddress>,
    pub token_expire_in: Duration,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ScoutReminders {
    pub not_updated_for: Duration,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OwnerReminders {
    pub not_updated_for: Duration,
}

impl TryFrom<raw::Config> for Config {
    type Error = anyhow::Error;
    fn try_from(from: raw::Config) -> anyhow::Result<Self> {
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
                let toml_name = gw_name.as_str();
                let gateway = gateway.clone().unwrap_or_default();

                let gw = match gw_name {
                    raw::EmailGateway::Mailgun => {
                        let raw::MailGun {
                            api_key,
                            api_base_url,
                            domain,
                            sender_address,
                        } = gateway.mailgun.ok_or_else(|| {
                            anyhow!("Missing '{toml_name}' gateway configuration")
                        })?;
                        let sender_address = sender_address.parse()?;
                        let api_base_url = api_base_url
                            .unwrap_or_else(|| "https://api.eu.mailgun.net/v3".to_string());
                        log::info!("Use Mailgun gateway");
                        EmailGateway::MailGun {
                            api_key,
                            api_base_url,
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
                let toml_name = gw_name.as_str();
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
            send_to,
            send_bcc,
            scouts,
            owners,
            token_expire_in,
        } = reminders.unwrap_or_default();

        let send_bcc = if let Some(bcc) = send_bcc {
            bcc.into_iter()
                .map(|a| a.parse::<EmailAddress>())
                .collect::<anyhow::Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        let default_reminders_cfg = raw::Reminders::default();

        let task_interval_time = task_interval_time.unwrap_or_else(|| {
            default_reminders_cfg
                .task_interval_time
                .expect("Reminder task interval")
        });
        let token_expire_in = token_expire_in.unwrap_or_else(|| {
            default_reminders_cfg
                .token_expire_in
                .expect("Token expire duration")
        });
        let send_max = send_max.unwrap_or_else(|| {
            default_reminders_cfg
                .send_max
                .expect("Send max. reminders configuration")
        });

        let send_to = send_to
            .unwrap_or_default()
            .into_iter()
            .map(RecipientRole::from)
            .collect();

        let raw::ScoutReminders { not_updated_for } = scouts.unwrap_or_default();
        let scouts = ScoutReminders { not_updated_for };

        let raw::OwnerReminders { not_updated_for } = owners.unwrap_or_default();
        let owners = OwnerReminders { not_updated_for };

        let reminders = Reminders {
            task_interval_time,
            send_max,
            send_to,
            send_bcc,
            scouts,
            owners,
            token_expire_in,
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

impl From<raw::RecipientRole> for RecipientRole {
    fn from(from: raw::RecipientRole) -> Self {
        match from {
            raw::RecipientRole::Scouts => Self::Scout,
            raw::RecipientRole::Owners => Self::Owner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_config() {
        let file = Path::new("");
        let cfg = Config::try_load_from_file_or_default(file).unwrap();
        assert_eq!(cfg, Config::default());
        assert!(cfg.reminders.send_to.is_empty());
        assert!(cfg.reminders.send_bcc.is_empty());
    }

    #[test]
    fn hide_api_key_of_geo_gateway() {
        let x = GeocodingGateway::OpenCage {
            api_key: "123".to_string(),
        };
        let d = format!("{x:?}");
        assert_eq!(r#"OpenCage { api_key: "***" }"#, d);
    }

    #[test]
    fn hide_api_key_of_mailgun_gateway() {
        let x = EmailGateway::MailGun {
            api_base_url: "x".to_string(),
            domain: "y".to_string(),
            sender_address: "z@example.com".parse().unwrap(),
            api_key: "123".to_string(),
        };
        let d = format!("{x:?}");
        assert_eq!(
            r#"MailGun { api_base_url: "x", api_key: "***", domain: "y", sender_address: EmailAddress { address: "z@example.com", display_name: None } }"#,
            d
        );
    }
}
