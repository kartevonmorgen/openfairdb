use std::{collections::HashSet, sync::Arc};

use ofdb_core::gateways::notify::{NotificationEvent, NotificationGateway, NotificationType};
use ofdb_entities::{category::*, email::*};

use crate::{email::EmailGateway, user_communication};

#[derive(Clone)]
pub struct Notify {
    email_gw: Arc<dyn EmailGateway + Send + Sync + 'static>,
    notify_on: HashSet<NotificationType>,
}

impl Notify {
    pub fn new<G>(gw: G, notify_on: HashSet<NotificationType>) -> Self
    where
        G: EmailGateway + Send + Sync + 'static,
    {
        Self {
            email_gw: Arc::new(gw),
            notify_on,
        }
    }

    fn skip(&self, ev: &NotificationEvent) -> bool {
        !self.notify_on.contains(&ev.kind())
    }
}

impl NotificationGateway for Notify {
    fn notify(&self, event: NotificationEvent) {
        use NotificationEvent as E;
        if self.skip(&event) {
            return;
        }
        match event {
            E::PlaceAdded {
                place,
                email_addresses,
                all_categories,
            } => {
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
                    log::info!(
                        "Sending e-mails to {} recipients after new place {} added",
                        email_addresses.len(),
                        place.id,
                    );
                    compose_and_send_emails(&*self.email_gw, email_addresses, &content);
                }
            }
            E::PlaceUpdated {
                place,
                email_addresses,
                all_categories,
            } => {
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
                    log::info!(
                        "Sending e-mails to {} recipients after place {} updated",
                        email_addresses.len(),
                        place.id
                    );
                    compose_and_send_emails(&*self.email_gw, email_addresses, &content);
                }
            }
            E::EventAdded {
                event,
                email_addresses,
            } => {
                let content = user_communication::event_created_email(event);
                {
                    log::info!(
                        "Sending e-mails to {} recipients after new event {} created",
                        email_addresses.len(),
                        event.id,
                    );
                    compose_and_send_emails(&*self.email_gw, email_addresses, &content);
                }
            }
            E::EventUpdated {
                event,
                email_addresses,
            } => {
                let content = user_communication::event_updated_email(event);
                {
                    log::info!(
                        "Sending e-mails to {} recipients after event {} updated",
                        email_addresses.len(),
                        event.id
                    );
                    compose_and_send_emails(&*self.email_gw, email_addresses, &content);
                }
            }
            E::UserRegistered {
                user,
                confirmation_url,
            } => {
                let content =
                    user_communication::user_registration_email(confirmation_url.as_ref());
                {
                    log::info!("Sending confirmation e-mail to user {}", user.email);
                    compose_and_send_emails(&*self.email_gw, &[user.email.clone()], &content);
                }
            }
            E::UserResetPasswordRequested { email_nonce } => {
                let url = format!(
                    "https://openfairdb.org/reset-password?token={}",
                    email_nonce.encode_to_string()
                );
                let content = user_communication::user_reset_password_email(&url);
                {
                    log::info!(
                        "Sending e-mail to {} after password reset requested",
                        email_nonce.email
                    );
                    compose_and_send_emails(
                        &*self.email_gw,
                        &[email_nonce.email.to_owned()],
                        &content,
                    );
                }
            }
            E::ReminderCreated { email, recipients } => {
                self.email_gw.compose_and_send(recipients, email);
            }
        }
    }
}

fn compose_and_send_emails(
    gw: &dyn EmailGateway,
    recipients: &[EmailAddress],
    email_content: &EmailContent,
) {
    gw.compose_and_send(recipients, email_content);
}
