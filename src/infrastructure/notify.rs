use crate::core::prelude::*;
#[cfg(all(not(test), feature = "email"))]
use ofdb_core::EmailGateway;
#[cfg(all(not(test), feature = "email"))]
use ofdb_gateways::sendmail;
#[cfg(feature = "email")]
use ofdb_gateways::user_communication;

#[cfg(all(not(test), feature = "email"))]
const FROM_ADDRESS: &str = "\"Karte von morgen\" <no-reply@kartevonmorgen.org>";

/// Don't actually send emails while running the tests or
/// if the `email` feature is disabled.
#[cfg(all(test, feature = "email"))]
fn compose_and_send_emails(recipients: &[String], subject: &str, body: &str) {
    debug!(
        "Would compose e-mails to {:?} with subject '{}' and body '{}'",
        recipients, subject, body
    );
}

#[cfg(all(not(test), feature = "email"))]
fn compose_and_send_emails(recipients: &[String], subject: &str, body: &str) {
    let gw = sendmail::SendmailGateway::new(Email::from(FROM_ADDRESS));
    // TODO: take &[Email] as argument
    let rec: Vec<_> = recipients.iter().cloned().map(Email::from).collect();
    gw.compose_and_send(&rec, subject, body);
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
        compose_and_send_emails(&[user.email.clone()], &content.subject, &content.body);
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
