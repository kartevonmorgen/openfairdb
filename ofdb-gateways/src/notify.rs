use crate::user_communication;
use ofdb_core::gateways::{email::EmailGateway, notify::NotificationGateway};
use ofdb_entities::{category::*, email::*, event::*, nonce::*, place::*, user::*};

pub struct Notify {
    email_gw: Box<dyn EmailGateway + Send + Sync + 'static>,
}

impl Notify {
    pub fn new<G>(gw: G) -> Self
    where
        G: EmailGateway + Send + Sync + 'static,
    {
        Self {
            email_gw: Box::new(gw),
        }
    }
}

impl NotificationGateway for Notify {
    fn place_added(
        &self,
        email_addresses: &[String],
        place: &Place,
        all_categories: Vec<Category>,
    ) {
        let mut place = place.clone();
        let (tags, categories) = Category::split_from_tags(place.tags);
        place.tags = tags;
        let category_names: Vec<String> = all_categories
            .into_iter()
            .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
            .map(|c| c.name())
            .collect();
        let content = user_communication::place_created_email(&place, &category_names);

        {
            info!(
                "Sending e-mails to {} recipients after new place {} added",
                email_addresses.len(),
                place.id,
            );
            compose_and_send_emails(
                &*self.email_gw,
                email_addresses,
                &content.subject,
                &content.body,
            );
        }
    }
    fn place_updated(
        &self,
        email_addresses: &[String],
        place: &Place,
        all_categories: Vec<Category>,
    ) {
        let mut place = place.clone();
        let (tags, categories) = Category::split_from_tags(place.tags);
        place.tags = tags;
        let category_names: Vec<String> = all_categories
            .into_iter()
            .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
            .map(|c| c.name())
            .collect();
        let content = user_communication::place_updated_email(&place, &category_names);

        {
            info!(
                "Sending e-mails to {} recipients after place {} updated",
                email_addresses.len(),
                place.id
            );
            compose_and_send_emails(
                &*self.email_gw,
                email_addresses,
                &content.subject,
                &content.body,
            );
        }
    }
    fn event_created(&self, email_addresses: &[String], event: &Event) {
        let content = user_communication::event_created_email(event);

        {
            info!(
                "Sending e-mails to {} recipients after new event {} created",
                email_addresses.len(),
                event.id,
            );
            compose_and_send_emails(
                &*self.email_gw,
                email_addresses,
                &content.subject,
                &content.body,
            );
        }
    }
    fn event_updated(&self, email_addresses: &[String], event: &Event) {
        let content = user_communication::event_updated_email(event);

        {
            info!(
                "Sending e-mails to {} recipients after event {} updated",
                email_addresses.len(),
                event.id
            );
            compose_and_send_emails(
                &*self.email_gw,
                email_addresses,
                &content.subject,
                &content.body,
            );
        }
    }
    fn user_registered_kvm(&self, user: &User) {
        let token = EmailNonce {
            email: user.email.clone(),
            nonce: Nonce::new(),
        }
        .encode_to_string();
        let url = format!("https://kartevonmorgen.org/#/?confirm_email={}", token);
        self.user_registered(user, &url);
    }
    fn user_registered_ofdb(&self, user: &User) {
        let token = EmailNonce {
            email: user.email.clone(),
            nonce: Nonce::new(),
        }
        .encode_to_string();
        let url = format!("https://openfairdb.org/register/confirm/{}", token);
        self.user_registered(user, &url);
    }
    fn user_registered(&self, user: &User, url: &str) {
        let content = user_communication::user_registration_email(url);

        {
            info!("Sending confirmation e-mail to user {}", user.email);
            compose_and_send_emails(
                &*self.email_gw,
                &[user.email.clone()],
                &content.subject,
                &content.body,
            );
        }
    }
    fn user_reset_password_requested(&self, email_nonce: &EmailNonce) {
        let url = format!(
            "https://openfairdb.org/reset-password?token={}",
            email_nonce.encode_to_string()
        );
        let content = user_communication::user_reset_password_email(&url);

        {
            info!(
                "Sending e-mail to {} after password reset requested",
                email_nonce.email
            );
            compose_and_send_emails(
                &*self.email_gw,
                &[email_nonce.email.to_owned()],
                &content.subject,
                &content.body,
            );
        }
    }
}

fn compose_and_send_emails(
    gw: &dyn EmailGateway,
    recipients: &[String],
    subject: &str,
    body: &str,
) {
    // TODO: take &[Email] as argument
    let rec: Vec<_> = recipients.iter().cloned().map(Email::from).collect();
    gw.compose_and_send(&rec, subject, body);
}
