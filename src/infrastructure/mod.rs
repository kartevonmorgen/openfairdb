pub mod db;
pub mod error;
pub mod flows;

use ofdb_entities::email::*;
use ofdb_gateways::{mailgun::*, opencage::*, sendmail::*};
use std::env;

lazy_static! {

    pub static ref GEO_CODING_GW: OpenCage = {
        let key = match env::var("OPENCAGE_API_KEY") {
            Ok(key) => Some(key),
            Err(_) => {
                warn!("No OpenCage API key found");
                None
            }
        };
        OpenCage::new(key)
    };

    pub static ref MAILGUN_GW: Option<Mailgun> = {
        let api_key = env::var("MAILGUN_API_KEY");
        let domain = env::var("MAILGUN_DOMAIN");
        let from = env::var("MAIL_GATEWAY_SENDER_ADDRESS");

        if let (Ok(api_key), Ok(mail), Ok(domain)) = (api_key, from, domain) {
            let api_url = env::var("MAILGUN_API_URL").unwrap_or_else(|_|format!("https://api.eu.mailgun.net/v3/{}/messages", domain));
            // TODO: validate values
            Some(Mailgun {
                from_email: Email::from(mail),
                domain,
                api_key,
                api_url,
            })
        } else {
            None
        }

    };

    pub static ref SENDMAIL_GW: Option<Sendmail> = {
        let from = env::var("MAIL_GATEWAY_SENDER_ADDRESS");
        if let Ok(mail) = from {
            // TODO: validate values
            Some(
                Sendmail::new(Email::from(mail)),
            )
        } else {
            None
        }
    };
}

#[cfg(test)]
mod tests;
