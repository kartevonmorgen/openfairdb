#[cfg(test)]
use crate::ports::web::tests::DummyNotifyGW;
use core::ops::Deref;
#[cfg(not(test))]
use ofdb_gateways::notify::NotificationGW;
use rocket::{
    request::{self, FromRequest},
    Outcome, Request,
};

#[cfg(not(test))]
pub struct Notify(NotificationGW);

#[cfg(test)]
pub struct Notify(DummyNotifyGW);

impl Deref for Notify {
    type Target = dyn ofdb_core::NotificationGateway;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Notify {
    type Error = ();

    #[cfg(not(test))]
    fn from_request(_: &'a Request<'r>) -> request::Outcome<Self, ()> {
        Outcome::Success(Notify(NotificationGW::new()))
    }
    #[cfg(test)]
    fn from_request(_: &'a Request<'r>) -> request::Outcome<Self, ()> {
        Outcome::Success(Notify(DummyNotifyGW))
    }
}
