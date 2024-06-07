use std::collections::HashSet;

use ofdb_core::{
    entities::{EmailAddress, EmailContent},
    gateways::{geocode::GeoCodingGateway, notify::NotificationType},
};
use ofdb_gateways::{
    email::{
        mailgun::Mailgun, send_to_json_file::SendToJsonFile, sendmail::Sendmail, EmailGateway,
    },
    notify::Notify,
    opencage::OpenCage,
};

use crate::config;

const ALLWAYS_NOTIFY_ON: [NotificationType; 2] = [
    NotificationType::UserRegistered,
    NotificationType::UserResetPasswordRequested,
];

pub fn notification_gateway(
    gateway_cfg: Option<config::EmailGateway>,
    subscriptions_cfg: config::Subscriptions,
) -> Notify {
    let notify_on = HashSet::from(ALLWAYS_NOTIFY_ON)
        .union(&subscriptions_cfg.notify_on)
        .copied()
        .collect();

    let Some(gateway_cfg) = gateway_cfg else {
        log::info!("No eMail gateway was configured");
        return Notify::new(DummyMailGw, notify_on);
    };

    match gateway_cfg {
        config::EmailGateway::MailGun {
            api_key,
            domain,
            sender_address,
            api_base_url,
        } => {
            let mailgun = Mailgun {
                from_email: sender_address,
                domain,
                api_key,
                api_base_url,
            };
            Notify::new(mailgun, notify_on)
        }
        config::EmailGateway::Sendmail { sender_address } => {
            let sendmail = Sendmail::new(sender_address);
            Notify::new(sendmail, notify_on)
        }
        config::EmailGateway::EmailToJsonFile { dir } => {
            let Ok(gw) = SendToJsonFile::try_new(dir).map_err(|err| {
                log::warn!("Could not create JSON file email gateway: {err}");
            }) else {
                return Notify::new(DummyMailGw, notify_on);
            };
            Notify::new(gw, notify_on)
        }
    }
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
