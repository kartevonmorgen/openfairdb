use crate::{adapters::user_communication, core::prelude::*};
#[cfg(feature = "email")]
use ofdb_gateways::sendmail;

#[cfg(all(not(test), feature = "email"))]
fn send_email(mail: String) {
    std::thread::spawn(move || {
        if let Err(err) = sendmail::send(&mail) {
            warn!("Could not send e-mail: {}", err);
        }
    });
}

/// Don't actually send emails while running the tests or
/// if the `email` feature is disabled.
#[cfg(all(test, feature = "email"))]
fn send_email(email: String) {
    debug!("Would send e-mail: {}", email);
}

#[cfg(feature = "email")]
pub fn compose_and_send_email(to: &str, subject: &str, body: &str) {
    match sendmail::compose(&[to], subject, body) {
        Ok(email) => send_email(email),
        Err(err) => {
            warn!("Failed to compose e-mail: {}", err);
        }
    }
}

#[cfg(feature = "email")]
pub fn compose_and_send_emails(recipients: &[String], subject: &str, body: &str) {
    debug!("Sending e-mails to: {:?}", recipients);
    for to in recipients {
        compose_and_send_email(to, subject, body);
    }
}

pub fn place_added(email_addresses: &[String], place: &Place, all_categories: Vec<Category>) {
    let mut place = place.clone();
    let (tags, categories) = Category::split_from_tags(place.tags);
    place.tags = tags;
    let category_names: Vec<String> = all_categories
        .into_iter()
        .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
        .map(|c| c.name())
        .collect();
    let content = user_communication::place_created_email(&place, &category_names);

    #[cfg(feature = "email")]
    {
        info!(
            "Sending e-mails to {} recipients after new place {} added",
            email_addresses.len(),
            place.id,
        );
        compose_and_send_emails(email_addresses, &content.subject, &content.body);
    }
}

pub fn place_updated(email_addresses: &[String], place: &Place, all_categories: Vec<Category>) {
    let mut place = place.clone();
    let (tags, categories) = Category::split_from_tags(place.tags);
    place.tags = tags;
    let category_names: Vec<String> = all_categories
        .into_iter()
        .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
        .map(|c| c.name())
        .collect();
    let content = user_communication::place_updated_email(&place, &category_names);

    #[cfg(feature = "email")]
    {
        info!(
            "Sending e-mails to {} recipients after place {} updated",
            email_addresses.len(),
            place.id
        );
        compose_and_send_emails(email_addresses, &content.subject, &content.body);
    }
}

pub fn event_created(email_addresses: &[String], event: &Event) {
    let content = user_communication::event_created_email(&event);

    #[cfg(feature = "email")]
    {
        info!(
            "Sending e-mails to {} recipients after new event {} created",
            email_addresses.len(),
            event.id,
        );
        compose_and_send_emails(email_addresses, &content.subject, &content.body);
    }
}

pub fn event_updated(email_addresses: &[String], event: &Event) {
    let content = user_communication::event_updated_email(&event);

    #[cfg(feature = "email")]
    {
        info!(
            "Sending e-mails to {} recipients after event {} updated",
            email_addresses.len(),
            event.id
        );
        compose_and_send_emails(email_addresses, &content.subject, &content.body);
    }
}

pub fn user_registered_kvm(user: &User) {
    let token = EmailNonce {
        email: user.email.clone(),
        nonce: Nonce::new(),
    }
    .encode_to_string();
    let url = format!("https://kartevonmorgen.org/#/?confirm_email={}", token);
    user_registered(user, &url);
}

pub fn user_registered_ofdb(user: &User) {
    let token = EmailNonce {
        email: user.email.clone(),
        nonce: Nonce::new(),
    }
    .encode_to_string();
    let url = format!("https://openfairdb.org/register/confirm/{}", token);
    user_registered(user, &url);
}

pub fn user_registered(user: &User, url: &str) {
    let content = user_communication::user_registration_email(&url);

    #[cfg(feature = "email")]
    {
        info!("Sending confirmation e-mail to user {}", user.email);
        compose_and_send_email(&user.email, &content.subject, &content.body);
    }
}

pub fn user_reset_password_requested(email_nonce: &EmailNonce) {
    let url = format!(
        "https://openfairdb.org/reset-password?token={}",
        email_nonce.encode_to_string()
    );
    let content = user_communication::user_reset_password_email(&url);

    #[cfg(feature = "email")]
    {
        info!(
            "Sending e-mail to {} after password reset requested",
            email_nonce.email
        );
        compose_and_send_emails(
            &[email_nonce.email.to_owned()],
            &content.subject,
            &content.body,
        );
    }
}
