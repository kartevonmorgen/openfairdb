use crate::{
    repositories,
    util::validate::{ContactInvalidation, EventInvalidation, PlaceInvalidation},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The title is invalid")]
    Title,
    #[error("Bounding box is invalid")]
    Bbox,
    #[error("Unsupported license")]
    License,
    #[error("Invalid email address")]
    Email,
    #[error("Invalid phone nr")]
    Phone,
    #[error("Invalid URL")]
    Url,
    #[error("Invalid contact")]
    Contact,
    #[error("Invalid registration type")]
    RegistrationType,
    #[error("The user already exists")]
    UserExists,
    #[error("The user does not exist")]
    UserDoesNotExist,
    #[error("Invalid password")]
    Password,
    #[error("Empty comment")]
    EmptyComment,
    #[error("Rating value out of range")]
    RatingValue,
    #[error("Invalid rating context")]
    RatingContext(String),
    #[error("Invalid credentials")]
    Credentials,
    #[error("Email not confirmed")]
    EmailNotConfirmed,
    #[error("This is not allowed")]
    Forbidden,
    #[error("This is not allowed without auth")]
    Unauthorized,
    #[error("The end date is before the start")]
    EndDateBeforeStart,
    #[error("The tag is owned by an organization")]
    ModeratedTag,
    #[error("Missing the email of the creator")]
    CreatorEmail,
    #[error("Invalid opening hours")]
    InvalidOpeningHours,
    #[error("Invalid position")]
    InvalidPosition,
    #[error("Invalid limit")]
    InvalidLimit,
    #[error("Token invalid")]
    TokenInvalid,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Missing id list")]
    EmptyIdList,
    #[error(transparent)]
    Repo(#[from] repositories::Error),
}

impl From<ofdb_entities::password::ParseError> for Error {
    fn from(_: ofdb_entities::password::ParseError) -> Self {
        Self::Password
    }
}

impl From<ofdb_entities::event::RegistrationTypeParseError> for Error {
    fn from(_: ofdb_entities::event::RegistrationTypeParseError) -> Self {
        Self::RegistrationType
    }
}

impl From<ofdb_entities::nonce::EmailNonceDecodingError> for Error {
    fn from(_: ofdb_entities::nonce::EmailNonceDecodingError) -> Self {
        Self::InvalidNonce
    }
}

impl From<ofdb_entities::url::ParseError> for Error {
    fn from(_: ofdb_entities::url::ParseError) -> Self {
        Self::Url
    }
}

impl From<PlaceInvalidation> for Error {
    fn from(err: PlaceInvalidation) -> Self {
        match err {
            PlaceInvalidation::License => Self::License,
            PlaceInvalidation::Contact(err) => err.into(),
        }
    }
}

impl From<EventInvalidation> for Error {
    fn from(err: EventInvalidation) -> Self {
        match err {
            EventInvalidation::Title => Self::Title,
            EventInvalidation::EndDateBeforeStart => Self::EndDateBeforeStart,
            EventInvalidation::Contact(err) => err.into(),
        }
    }
}

impl From<ContactInvalidation> for Error {
    fn from(err: ContactInvalidation) -> Self {
        match err {
            ContactInvalidation::Email => Self::Email,
        }
    }
}
