use ofdb_entities::email::Email;

pub trait EmailGateway {
    fn compose_and_send(&self, recipients: &[Email], subject: &str, body: &str);
}
