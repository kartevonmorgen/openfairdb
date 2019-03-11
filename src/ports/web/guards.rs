use crate::core::entities::Role;
use num_traits::FromPrimitive;
use rocket::{
    self,
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
    Outcome,
};

pub const COOKIE_EMAIL_KEY: &str = "ofdb-user-email";
pub const COOKIE_USER_KEY: &str = "user_id";
pub const COOKIE_USER_ACCESS_LEVEL: &str = "ofdb-user-access-level";

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

#[derive(Debug)]
pub struct Admin(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for Admin {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Admin, ()> {
        let user = request
            .cookies()
            .get_private(COOKIE_EMAIL_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Account);
        let role = request
            .cookies()
            .get_private(COOKIE_USER_ACCESS_LEVEL)
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(Role::from_usize);

        match (user, role) {
            (Some(user), Some(role)) => {
                if role == Role::Admin {
                    Some(Admin(user.0))
                } else {
                    return Outcome::Failure((Status::Unauthorized, ()));
                }
            }
            _ => None,
        }
        .or_forward(())
    }
}
