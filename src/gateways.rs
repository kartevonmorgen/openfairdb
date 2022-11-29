use crate::config;
use ofdb_core::{
    entities::{EmailAddress, EmailContent},
    gateways::email::EmailGateway,
    gateways::geocode::GeoCodingGateway,
};
use ofdb_gateways::{
    email::{mailgun::Mailgun, send_to_json_file::SendToJsonFile, sendmail::Sendmail},
    notify::Notify,
    opencage::OpenCage,
};

pub fn notification_gateway(cfg: Option<config::EmailGateway>) -> Notify {
    cfg.and_then(|gw| match gw {
        config::EmailGateway::MailGun {
            api_key,
            domain,
            sender_address,
            api_url,
        } => Some(Notify::new(Mailgun {
            from_email: sender_address,
            domain,
            api_key,
            api_url,
        })),
        config::EmailGateway::Sendmail { sender_address } => {
            Some(Notify::new(Sendmail::new(sender_address)))
        }
        config::EmailGateway::EmailToJsonFile { dir } => SendToJsonFile::try_new(dir)
            .map_err(|err| {
                log::warn!("Could not create JSON file email gateway: {err}");
            })
            .ok()
            .map(Notify::new),
    })
    .unwrap_or_else(|| Notify::new(DummyMailGw))
}

pub fn email_gateway(cfg: Option<config::EmailGateway>) -> EmailGw {
    cfg.and_then(|gw| match gw {
        config::EmailGateway::MailGun {
            api_key,
            domain,
            sender_address,
            api_url,
        } => Some(EmailGw::new(Mailgun {
            from_email: sender_address,
            domain,
            api_key,
            api_url,
        })),
        config::EmailGateway::Sendmail { sender_address } => {
            Some(EmailGw::new(Sendmail::new(sender_address)))
        }
        config::EmailGateway::EmailToJsonFile { dir } => SendToJsonFile::try_new(dir)
            .map_err(|err| {
                log::warn!("Could not create JSON file email gateway: {err}");
            })
            .ok()
            .map(EmailGw::new),
    })
    .unwrap_or_else(|| {
        log::info!("No eMail gateway was configured");
        EmailGw::new(DummyMailGw)
    })
}

pub fn geocoding_gateway(
    cfg: Option<config::GeocodingGateway>,
) -> Box<dyn GeoCodingGateway + Send + Sync> {
    match cfg {
        Some(config::GeocodingGateway::OpenCage { api_key }) => {
            Box::new(OpenCage::new(Some(api_key)))
        }
        _ => Box::new(NoGeoCodingGateway),
    }
}

struct NoGeoCodingGateway;

impl GeoCodingGateway for NoGeoCodingGateway {
    fn resolve_address_lat_lng(&self, _: &ofdb_core::entities::Address) -> Option<(f64, f64)> {
        None
    }
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
