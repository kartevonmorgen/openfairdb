use core::ops::Deref;

use ofdb_core::gateways::email::EmailGateway;
use ofdb_entities::email::*;
#[cfg(not(test))]
use ofdb_gateways::notify;
use rocket::{
    request::{self, FromRequest},
    Outcome, Request,
};

#[cfg(not(test))]
use crate::infrastructure::{MAILGUN_GW, SENDMAIL_GW};
#[cfg(test)]
use crate::ports::web::tests::DummyNotifyGW;

#[cfg(not(test))]
pub struct Notify(notify::Notify);

#[cfg(test)]
pub struct Notify(DummyNotifyGW);

struct DummyMailGw;

impl EmailGateway for DummyMailGw {
    fn compose_and_send(&self, _recipients: &[Email], _subject: &str, _body: &str) {
        debug!("Cannot send emails because no e-mail gateway was configured");
    }
}

impl Deref for Notify {
    type Target = dyn ofdb_core::gateways::notify::NotificationGateway;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Notify {
    type Error = ();

    #[cfg(not(test))]
    fn from_request(_: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if let Some(gw) = &*MAILGUN_GW {
            info!("Use Mailgun gateway");
            Outcome::Success(Notify(notify::Notify::new(gw.clone())))
        } else if let Some(gw) = &*SENDMAIL_GW {
            warn!("Mailgun gateway was not configured: use sendmail as fallback");
            Outcome::Success(Notify(notify::Notify::new(gw.clone())))
        } else {
            warn!("No eMail gateway was not configured");
            Outcome::Success(Notify(notify::Notify::new(DummyMailGw)))
        }
    }
    #[cfg(test)]
    fn from_request(_: &'a Request<'r>) -> request::Outcome<Self, ()> {
        Outcome::Success(Notify(DummyNotifyGW))
    }
}
