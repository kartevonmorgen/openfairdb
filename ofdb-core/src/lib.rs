#![feature(test)]
#[cfg(test)]
extern crate test;

use ofdb_entities::{address::*, category::*, email::*, event::*, nonce::*, place::*, user::*};

pub mod authorization;
pub mod util;

pub trait EmailGateway {
    fn compose_and_send(&self, recipients: &[Email], subject: &str, body: &str);
}

pub trait NotificationGateway {
    fn place_added(&self, email_addresses: &[String], place: &Place, all_categories: Vec<Category>);
    fn place_updated(
        &self,
        email_addresses: &[String],
        place: &Place,
        all_categories: Vec<Category>,
    );
    fn event_created(&self, email_addresses: &[String], event: &Event);
    fn event_updated(&self, email_addresses: &[String], event: &Event);
    fn user_registered_kvm(&self, user: &User);
    fn user_registered_ofdb(&self, user: &User);
    fn user_registered(&self, user: &User, url: &str);
    fn user_reset_password_requested(&self, email_nonce: &EmailNonce);
}

pub trait GeoCodingGateway {
    fn resolve_address_lat_lng(&self, addr: &Address) -> Option<(f64, f64)>;
}
