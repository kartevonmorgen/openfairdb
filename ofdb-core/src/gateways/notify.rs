use ofdb_entities::{
    category::Category, email::EmailAddress, event::Event, nonce::EmailNonce, place::Place,
    user::User,
};

pub trait NotificationGateway {
    fn place_added(
        &self,
        email_addresses: &[EmailAddress],
        place: &Place,
        all_categories: Vec<Category>,
    );
    fn place_updated(
        &self,
        email_addresses: &[EmailAddress],
        place: &Place,
        all_categories: Vec<Category>,
    );
    fn event_created(&self, email_addresses: &[EmailAddress], event: &Event);
    fn event_updated(&self, email_addresses: &[EmailAddress], event: &Event);
    fn user_registered_kvm(&self, user: &User);
    fn user_registered_ofdb(&self, user: &User);
    fn user_registered(&self, user: &User, url: &str /* TODO: use Url */);
    fn user_reset_password_requested(&self, email_nonce: &EmailNonce);
}
