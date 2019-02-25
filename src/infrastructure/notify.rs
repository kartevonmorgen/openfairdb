#[cfg(feature = "email")]
use super::mail;
use crate::{adapters::user_communication, core::prelude::*};

#[cfg(all(not(test), feature = "email"))]
fn send_mail(mail: String) {
    std::thread::spawn(move || {
        if let Err(err) = mail::sendmail::send(&mail) {
            warn!("Could not send e-mail: {}", err);
        }
    });
}

/// Don't actually send emails while running the tests
#[cfg(all(test, feature = "email"))]
fn send_mail(mail: String) {
    debug!("Would send e-mail: {}", mail);
}

#[cfg(feature = "email")]
pub fn send_mails(email_addresses: &[String], subject: &str, body: &str) {
    debug!("sending emails to: {:?}", email_addresses);
    for email_address in email_addresses.to_owned() {
        let to = vec![email_address];
        match mail::create(&to, subject, body) {
            Ok(mail) => {
                send_mail(mail);
            }
            Err(e) => {
                warn!("could not create notification mail: {}", e);
            }
        }
    }
}

pub fn entry_added(email_addresses: &[String], entry: &Entry, all_categories: Vec<Category>) {
    let subject = format!("Karte von morgen - neuer Eintrag: {}", entry.title);
    let category_names: Vec<String> = all_categories
        .into_iter()
        .filter(|c| entry.categories.iter().any(|c_id| &c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::new_entry_email(entry, &category_names);

    #[cfg(feature = "email")]
    send_mails(email_addresses, &subject, &body);
}

pub fn entry_updated(email_addresses: &[String], entry: &Entry, all_categories: Vec<Category>) {
    let subject = format!("Karte von morgen - Eintrag verändert: {}", entry.title);
    let category_names: Vec<String> = all_categories
        .into_iter()
        .filter(|c| entry.categories.iter().any(|c_id| &c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::changed_entry_email(entry, &category_names);

    #[cfg(feature = "email")]
    send_mails(email_addresses, &subject, &body);
}
