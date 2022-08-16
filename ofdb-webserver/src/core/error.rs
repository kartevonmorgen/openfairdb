use ofdb_core::usecases::Error as ParameterError;

use thiserror::Error;

pub use ofdb_core::repositories;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parameter(#[from] ParameterError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Repo(#[from] repositories::Error),
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
