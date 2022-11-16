use ofdb_entities::email::*;

pub trait EmailGateway {
    // TODO: Make this async
    // TODO: Take vector of emails.
    fn compose_and_send(&self, recipients: &[EmailAddress], email: &EmailContent);
}
