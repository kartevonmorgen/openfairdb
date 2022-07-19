use ofdb_core::util::validate::{ContactInvalidation, EventInvalidation, PlaceInvalidation};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParameterError {
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
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("The requested object could not be found")]
    NotFound,
    #[cfg(test)]
    #[error("The object already exists")]
    AlreadyExists,
    #[error("The version of the object is invalid")]
    InvalidVersion,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parameter(#[from] ParameterError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error(transparent)]
    Pwhash(#[from] pwhash::error::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Internal(s)
    }
}

impl From<ofdb_entities::password::ParseError> for Error {
    fn from(_: ofdb_entities::password::ParseError) -> Self {
        Error::Parameter(ParameterError::Password)
    }
}

impl From<ofdb_entities::event::RegistrationTypeParseError> for Error {
    fn from(_: ofdb_entities::event::RegistrationTypeParseError) -> Self {
        Error::Parameter(ParameterError::RegistrationType)
    }
}

impl From<ofdb_entities::nonce::EmailNonceDecodingError> for Error {
    fn from(_: ofdb_entities::nonce::EmailNonceDecodingError) -> Self {
        Error::Parameter(ParameterError::InvalidNonce)
    }
}

impl From<ofdb_entities::url::ParseError> for Error {
    fn from(_: ofdb_entities::url::ParseError) -> Self {
        Error::Parameter(ParameterError::Url)
    }
}

impl From<PlaceInvalidation> for Error {
    fn from(err: PlaceInvalidation) -> Self {
        match err {
            PlaceInvalidation::License => ParameterError::License,
            PlaceInvalidation::Contact(err) => err.into(),
        }
        .into()
    }
}

impl From<EventInvalidation> for Error {
    fn from(err: EventInvalidation) -> Self {
        match err {
            EventInvalidation::Title => ParameterError::Title,
            EventInvalidation::EndDateBeforeStart => ParameterError::EndDateBeforeStart,
            EventInvalidation::Contact(err) => err.into(),
        }
        .into()
    }
}

impl From<ContactInvalidation> for ParameterError {
    fn from(err: ContactInvalidation) -> Self {
        match err {
            ContactInvalidation::Email => ParameterError::Email,
        }
    }
}
