use chrono::prelude::*;
use rocket::{
    self,
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
    Outcome,
};
use std::time::Duration;

pub const COOKIE_EMAIL_KEY: &str = "ofdb-user-email";
pub const COOKIE_USER_KEY: &str = "user_id";
pub const COOKIE_CAPTCHA_KEY: &str = "ofdb-captcha";

#[derive(Debug)]
pub struct Bearer(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for Bearer {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        match request.headers().get_one("Authorization") {
            Some(auth) => {
                let x: Vec<_> = auth.split(' ').collect();
                if x.len() != 2 {
                    return Outcome::Failure((Status::Unauthorized, ()));
                }
                if x[0] != "Bearer" {
                    return Outcome::Failure((Status::Unauthorized, ()));
                }
                Outcome::Success(Bearer(x[1].into()))
            }
            None => Outcome::Forward(()),
        }
    }
}

//TODO: remove and use Account instead
#[derive(Debug)]
pub struct Login(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for Login {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Login, ()> {
        let user = request
            .cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Login);
        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

#[derive(Debug)]
pub struct Account(String);

impl Account {
    pub fn email(&self) -> &str {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Account {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Account, !> {
        request
            .cookies()
            .get_private(COOKIE_EMAIL_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Account)
            .or_forward(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Captcha;

pub const MAX_CAPTCHA_TTL: Duration = Duration::from_secs(120);

impl<'a, 'r> FromRequest<'a, 'r> for Captcha {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let valid = request
            .cookies()
            .get_private(COOKIE_CAPTCHA_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|ts: DateTime<Utc>| Utc::now().signed_duration_since(ts).to_std().ok())
            .map(|duration: Duration| duration <= MAX_CAPTCHA_TTL)
            .unwrap_or(false);
        if valid {
            Outcome::Success(Captcha)
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

#[derive(Debug, Default)]
pub struct Auth {
    account: Option<String>,
    bearer: Option<String>,
}

impl Auth {
    pub fn email(&self) -> Option<&str> {
        self.account.as_deref()
    }
    pub fn bearer(&self) -> Option<&str> {
        self.bearer.as_deref()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let mut auth = Auth::default();
        if let Outcome::Success(b) = Bearer::from_request(req) {
            auth.bearer = Some(b.0);
        }
        if let Outcome::Success(a) = Account::from_request(req) {
            auth.account = Some(a.0);
        }
        let captcha = Outcome::Success(Captcha) == Captcha::from_request(req);
        if !captcha && auth.account.is_none() && auth.bearer.is_none() {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(auth)
    }
}
