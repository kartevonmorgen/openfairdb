use std;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use toml;
use super::error::AppError;
use fast_chemail::is_valid_email;

type Result<T> = std::result::Result<T,AppError>;

mod raw {

    #[derive(Deserialize)]
    pub struct Config {
        pub notification: Option<NotificationCfg>
    }

    #[derive(Deserialize)]
    pub struct NotificationCfg {
        #[serde(rename = "send-to")]
        pub send_to: Option<Vec<String>>
    }
}

pub struct Config {
    pub notification: NotificationCfg
}

pub struct NotificationCfg {
    pub send_to: Vec<String>
}

impl Config {
    pub fn load<P>(file_name: P) -> Result<Config>
        where P: AsRef<Path>
     {
        match File::open(file_name) {
           Ok(mut f) => {
                let mut toml_str = String::new();
                f.read_to_string(&mut toml_str)?;
                from_str(&toml_str)
           }
           Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!("No configuration file found");
                    return Ok(Config::default());
                }
                return Err(err.into())
            }
        }
    }
}

fn from_str(toml: &str) -> Result<Config> {
    let raw: raw::Config = toml::from_str(toml)?;
    let mut cfg = Config::default();
    if let Some(n) = raw.notification {
        if let Some(mails) = n.send_to {
            let mails = mails.into_iter()
                             .filter(|m|is_valid_email(m))
                             .collect();
            cfg.notification.send_to = mails
        }
    }
    Ok(cfg)
}

impl Default for Config {
    fn default() -> Config {
        Config {
            notification: NotificationCfg {
                send_to: vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;


   #[test]
   fn define_notification_mail_addresses() {
       let s = r#"
           [notification]
           send-to = ["foo@bar.baz", "baz@blub.org"]
           "#;
       let emails = from_str(s).unwrap().notification.send_to;
       assert_eq!(emails[0], "foo@bar.baz");
       assert_eq!(emails[1], "baz@blub.org");
   }

   #[test]
   fn ignore_invalid_notification_mail_addresses() {
       let s = r#"
           [notification]
           send-to = ["not-valid", "valid@foo.bar"]
           "#;
       let emails = from_str(s).unwrap().notification.send_to;
       assert_eq!(emails.len(), 1);
       assert_eq!(emails[0], "valid@foo.bar");
   }
}
