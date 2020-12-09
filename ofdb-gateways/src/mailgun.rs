use ofdb_core::gateways::email::EmailGateway;
use ofdb_entities::email::*;
#[cfg(not(test))]
use std::io::{Error, ErrorKind};
use std::{io::Result, thread};

/// An email notification manager based on mailgun.net.
#[derive(Debug, Clone)]
pub struct Mailgun {
    pub api_key: String,
    pub api_url: String,
    pub domain: String,
    pub from_email: Email,
}

impl Mailgun {
    fn send(&self, params: Vec<(&'static str, String)>) {
        let url = self.api_url.clone();
        let key = self.api_key.clone();
        thread::spawn(move || {
            if let Err(err) = send_raw(&url, &key, params) {
                warn!("Could not send e-mail: {}", err);
            }
        });
    }
}

#[cfg(not(test))]
fn send_raw(url: &str, api_key: &str, params: Vec<(&'static str, String)>) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .form(&params)
        .basic_auth("api", Some(api_key))
        .send();
    res.map_err(|err| Error::new(ErrorKind::Other, err))
        .and_then(|res| {
            if res.status().is_success() {
                debug!("Mail provider response: {:#?}", res);
                Ok(())
            } else {
                // TODO: We should distinguish between a technical failure (Err, see above)
                // and an error response (here).
                Err(Error::new(
                    ErrorKind::Other,
                    format!("Could not send email: response status: {:?}", res.status()),
                ))
            }
        })
}

/// Don't actually send emails while running the tests or
/// if the `email` feature is disabled.
#[cfg(test)]
fn send_raw(_: &str, _: &str, params: Vec<(&'static str, String)>) -> Result<()> {
    debug!("Would send e-mail: {:?}", params);
    Ok(())
}

impl EmailGateway for Mailgun {
    fn compose_and_send(&self, recipients: &[Email], subject: &str, body: &str) {
        if recipients.is_empty() {
            warn!("No valid email adresses specified");
            return;
        }
        debug!("Sending e-mails to: {:?}", recipients);
        let recipients: String = recipients
            .iter()
            .map(std::ops::Deref::deref)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");

        let params = vec![
            ("from", (*self.from_email).clone()),
            ("bcc", recipients),
            ("subject", subject.to_owned()),
            ("text", body.to_owned()),
        ];
        self.send(params);
    }
}
