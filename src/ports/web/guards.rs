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

#[derive(Debug)]
pub struct Account(String);

impl Account {
    pub fn email(&self) -> &str {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Account {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Account, ()> {
        request
            .cookies()
            .get_private(COOKIE_EMAIL_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Account)
            .into_outcome((Status::Unauthorized, ()))
    }
}

#[derive(Debug, PartialEq)]
pub struct Captcha;

pub const MAX_CAPTCHA_TTL: Duration = Duration::from_secs(120);

impl<'a, 'r> FromRequest<'a, 'r> for Captcha {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        request
            .cookies()
            .get_private(COOKIE_CAPTCHA_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|ts: DateTime<Utc>| Utc::now().signed_duration_since(ts).to_std().ok())
            .and_then(|duration: Duration| {
                if duration <= MAX_CAPTCHA_TTL {
                    Some(Captcha)
                } else {
                    None
                }
            })
            .into_outcome((Status::Unauthorized, ()))
    }
}

#[derive(Debug, Default)]
pub struct Credentials {
    account_email: Option<String>,
    bearer_token: Option<String>,
}

impl Credentials {
    pub fn account_email(&self) -> Option<&str> {
        self.account_email.as_deref()
    }
    pub fn bearer_token(&self) -> Option<&str> {
        self.bearer_token.as_deref()
    }
    pub fn is_empty(&self) -> bool {
        self.account_email
            .as_deref()
            .map(str::is_empty)
            .unwrap_or(true)
            || self
                .bearer_token
                .as_deref()
                .map(str::is_empty)
                .unwrap_or(true)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Credentials {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let account_email = match Bearer::from_request(req) {
            Outcome::Success(account_email) => Some(account_email.0),
            _ => None,
        };
        let bearer_token = match Bearer::from_request(req) {
            Outcome::Success(bearer_token) => Some(bearer_token.0),
            _ => None,
        };
        let credentials = Credentials {
            account_email,
            bearer_token,
        };
        if credentials.is_empty() && Captcha::from_request(req) != Outcome::Success(Captcha) {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(credentials)
    }
}
