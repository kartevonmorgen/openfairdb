use crate::cfg::Cfg;
use ofdb_core::{
    entities::{EmailAddress, EmailContent},
    gateways::email::EmailGateway,
};
use ofdb_gateways::{mailgun::Mailgun, notify::Notify, opencage::OpenCage, sendmail::Sendmail};
use std::env;

pub fn notification_gateway() -> Notify {
    if let Some(gw) = mailgun_gw() {
        log::info!("Use Mailgun gateway");
        Notify::new(gw)
    } else if let Some(gw) = sendmail_gw() {
        log::warn!("Mailgun gateway was not configured: use sendmail as fallback");
        Notify::new(gw)
    } else {
        log::warn!("No eMail gateway was not configured");
        Notify::new(DummyMailGw)
    }
}

pub fn email_gateway() -> EmailGw {
    if let Some(gw) = mailgun_gw() {
        EmailGw::new(gw)
    } else if let Some(gw) = sendmail_gw() {
        EmailGw::new(gw)
    } else {
        EmailGw::new(DummyMailGw)
    }
}

pub fn geocoding_gateway(cfg: &Cfg) -> OpenCage {
    OpenCage::new(cfg.opencage_api_key.clone())
}

fn mailgun_gw() -> Option<Mailgun> {
    // TODO: move this to crate::cfg
    let api_key = env::var("MAILGUN_API_KEY");
    let domain = env::var("MAILGUN_DOMAIN");
    let from = env::var("MAIL_GATEWAY_SENDER_ADDRESS");

    let (Ok(api_key), Ok(mail), Ok(domain)) = (api_key, from, domain) else {
        return None;
    };

    // TODO: move this to crate::cfg
    let api_url = env::var("MAILGUN_API_URL")
        .unwrap_or_else(|_| format!("https://api.eu.mailgun.net/v3/{}/messages", domain));
    // TODO: validate values
    let Ok(from_email) = mail.parse() else {
        log::warn!("Invalid email set for 'MAIL_GATEWAY_SENDER_ADDRESS'");
        return None;
    };
    Some(Mailgun {
        from_email,
        domain,
        api_key,
        api_url,
    })
}

fn sendmail_gw() -> Option<Sendmail> {
    env::var("MAIL_GATEWAY_SENDER_ADDRESS")
        .ok()
        .and_then(|mail| {
            mail.parse()
                .map_err(|_| {
                    log::warn!("Invalid email set for 'MAIL_GATEWAY_SENDER_ADDRESS'");
                })
                .ok()
        })
        .map(Sendmail::new)
}

struct DummyMailGw;

impl EmailGateway for DummyMailGw {
    fn compose_and_send(&self, _recipients: &[EmailAddress], _email: &EmailContent) {
        log::debug!("Cannot send emails because no e-mail gateway was configured");
    }
}

pub struct EmailGw(Box<dyn EmailGateway + Send + Sync + 'static>);

impl EmailGw {
    pub fn new<G>(gw: G) -> Self
    where
        G: EmailGateway + Send + Sync + 'static,
    {
        Self(Box::new(gw))
    }
}

impl EmailGateway for EmailGw {
    fn compose_and_send(&self, recipients: &[EmailAddress], email: &EmailContent) {
        self.0.compose_and_send(recipients, email);
    }
}
