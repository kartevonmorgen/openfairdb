use anyhow::Result;
use itertools::Itertools;
use ofdb_entities::email::*;
use std::thread;

use super::EmailGateway;

/// An email notification manager based on mailgun.net.
#[derive(Debug, Clone)]
pub struct Mailgun {
    pub api_key: String,
    pub api_base_url: String,
    pub domain: String,
    pub from_email: EmailAddress,
}

impl Mailgun {
    fn send(&self, params: Vec<(&'static str, String)>) {
        let Self {
            api_base_url,
            domain,
            api_key,
            ..
        } = &self;
        // TODO: use url::Url
        let url = format!("{api_base_url}/{domain}/messages");
        let key = api_key.clone();
        // TODO: use tokio::task::spawn_blocking
        thread::spawn(move || {
            if let Err(err) = send_raw(&url, &key, params) {
                log::warn!("Could not send e-mail: {}", err);
            }
        });
    }
}

#[derive(Debug, serde::Deserialize, thiserror::Error)]
#[error("{message}")]
struct JsonError {
    pub message: String,
}

#[cfg(not(test))]
fn send_raw(url: &str, api_key: &str, params: Vec<(&'static str, String)>) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .form(&params)
        .basic_auth("api", Some(api_key))
        .send()?;
    if response.status().is_success() {
        log::debug!("Mail provider response: {:#?}", response);
        Ok(())
    } else {
        let json_error: JsonError = response.json()?;
        Err(json_error.into())
    }
}

/// Don't actually send emails while running the tests or
/// if the `email` feature is disabled.
#[cfg(test)]
fn send_raw(_: &str, _: &str, params: Vec<(&'static str, String)>) -> Result<()> {
    log::debug!("Would send e-mail: {:?}", params);
    Ok(())
}

impl EmailGateway for Mailgun {
    fn compose_and_send(&self, recipients: &[EmailAddress], email: &EmailContent) {
        if recipients.is_empty() {
            log::warn!("No valid email addresses specified");
            return;
        }
        log::debug!(
            "Sending e-mails from {} to: {:?}",
            self.from_email,
            recipients
        );
        let recipients: String = recipients.iter().map(EmailAddress::as_str).join(",");

        let params = vec![
            ("to", self.from_email.as_str().to_owned()), // `to` is required
            ("from", self.from_email.as_str().to_owned()),
            ("bcc", recipients),
            ("subject", email.subject.to_owned()),
            ("text", email.body.to_owned()),
        ];
        self.send(params);
    }
}
