use jfs::Store;
use ofdb_core::{entities::Timestamp, gateways::email::EmailGateway};
use ofdb_entities::email::*;
use serde::{Deserialize, Serialize};
use std::{io, path::Path};

/// A dummy email gateway for testing purposes.
pub struct SendToJsonFile {
    json_store: Store,
}

impl SendToJsonFile {
    pub fn try_new<P: AsRef<Path>>(directory: P) -> io::Result<Self> {
        let json_store = Store::new(directory)?;
        Ok(Self { json_store })
    }
    pub fn path(&self) -> &Path {
        self.json_store.path()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonEmail {
    to: String,
    subject: String,
    body: String,
}

impl JsonEmail {
    fn new(to: &EmailAddress, content: &EmailContent) -> Self {
        let subject = content.subject.to_owned();
        let body = content.body.to_owned();
        let to = to.as_str().to_owned();
        Self { to, subject, body }
    }
}

impl EmailGateway for SendToJsonFile {
    fn compose_and_send(&self, recipients: &[EmailAddress], content: &EmailContent) {
        let json_store = self.json_store.clone();
        let recipients = recipients.to_vec();
        let content = content.clone();
        for to in recipients {
            let now = Timestamp::now().as_millis();
            let key = format!("{now}-{to}");
            let email = JsonEmail::new(&to, &content);
            if let Err(err) = json_store.save_with_id(&email, &key) {
                log::warn!("Unable to save email in JSON file: {err}");
            }
        }
    }
}
