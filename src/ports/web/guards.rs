use std::time::Duration;

use chrono::prelude::*;
use rocket::{
    self,
    http::Status,
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
    State,
};

use crate::{
    core::{db::OrganizationRepo, prelude::*, usecases},
    infrastructure::error::AppError,
    ports::web::jwt,
};

pub const COOKIE_EMAIL_KEY: &str = "ofdb-user-email";
pub const COOKIE_CAPTCHA_KEY: &str = "ofdb-captcha";
pub const MAX_CAPTCHA_TTL: Duration = Duration::from_secs(120);

type Result<T> = std::result::Result<T, AppError>;

fn get_bearer_token(auth_header_val: &str) -> Option<&str> {
    let x: Vec<_> = auth_header_val.split(' ').collect();
    if x.len() == 2 && x[0] == "Bearer" {
        Some(x[1])
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Auth {
    bearer_tokens: Vec<String>,
    account_email: Option<String>,
    has_captcha: bool,
}

impl Auth {
    // pub fn unauthorized() -> AppError {
    //     AppError::Business(Error::Parameter(ParameterError::Unauthorized))
    // }

    pub fn account_email(&self) -> Result<&str> {
        self.account_email
            .as_deref()
            .ok_or(AppError::Business(Error::Parameter(
                ParameterError::Unauthorized,
            )))
    }

    pub fn bearer_tokens(&self) -> &Vec<String> {
        &self.bearer_tokens
    }

    pub fn has_captcha(&self) -> Result<()> {
        if self.has_captcha {
            Ok(())
        } else {
            Err(AppError::Business(Error::Parameter(
                ParameterError::Unauthorized,
            )))
        }
    }

    pub fn organization<R: OrganizationRepo>(&self, db: &R) -> Result<Organization> {
        Ok(usecases::authorize_organization_by_possible_api_tokens(
            db,
            &self.bearer_tokens,
        )?)
    }

    pub fn user_with_min_role<D: Db>(&self, db: &D, min_required_role: Role) -> Result<User> {
        Ok(usecases::authorize_user_by_email(
            db,
            self.account_email()?,
            min_required_role,
        )?)
    }

    fn bearer_tokens_from_header(request: &Request) -> Vec<String> {
        request
            .headers()
            .get("Authorization")
            .filter_map(get_bearer_token)
            .map(ToOwned::to_owned)
            .collect()
    }

    fn account_email_from_cookie(request: &Request) -> Option<String> {
        request
            .cookies()
            .get_private(COOKIE_EMAIL_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
    }

    async fn account_email_from_jwt_in_header(
        request: &Request<'_>,
        bearer_tokens: &[String],
    ) -> Option<String> {
        let jwt_state = request.guard::<&State<jwt::JwtState>>().await.succeeded()?;
        bearer_tokens
            .iter()
            .filter_map(|token| jwt_state.validate_token_and_get_email(token).ok())
            .next()
    }

    fn captcha_from_cookie(request: &Request) -> bool {
        request
            .cookies()
            .get_private(COOKIE_CAPTCHA_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .and_then(|ts: DateTime<Utc>| Utc::now().signed_duration_since(ts).to_std().ok())
            .map_or(false, |duration: Duration| duration <= MAX_CAPTCHA_TTL)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let bearer_tokens = Self::bearer_tokens_from_header(request);

        // decide account_email source
        let mut account_email = None;
        if cfg!(feature = "cookies") {
            account_email = Self::account_email_from_cookie(request);
        }
        if cfg!(feature = "jwt") && account_email.is_none() {
            account_email = Self::account_email_from_jwt_in_header(request, &bearer_tokens).await;
        }

        let has_captcha = Self::captcha_from_cookie(request);

        let auth = Self {
            bearer_tokens,
            account_email,
            has_captcha,
        };

        Outcome::Success(auth)
    }
}

#[derive(Debug)]
pub struct Account(String);

impl Account {
    pub fn email(&self) -> &str {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Account {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let auth = try_outcome!(Auth::from_request(request).await);
        match auth.account_email() {
            Ok(email) => Outcome::Success(Account(email.to_owned())),
            _ => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}
