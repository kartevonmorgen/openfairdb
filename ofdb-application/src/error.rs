use ofdb_core::{repositories::Error as RepoError, usecases::Error as ParameterError};
use std::io;
use thiserror::Error;

pub use ofdb_core::repositories;

impl From<RepoError> for AppError {
    fn from(err: RepoError) -> AppError {
        AppError::Business(BError::Repo(err))
    }
}

impl From<ofdb_core::usecases::Error> for AppError {
    fn from(err: ofdb_core::usecases::Error) -> AppError {
        AppError::Business(err.into())
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Business(#[from] BError),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    R2d2(#[from] r2d2::Error),
    #[error(transparent)]
    String(#[from] ::std::string::FromUtf8Error),
    #[error(transparent)]
    Str(#[from] ::std::str::Utf8Error),
    #[error(transparent)]
    Csv(#[from] ::csv::Error),
}

impl From<ofdb_entities::password::ParseError> for AppError {
    fn from(err: ofdb_entities::password::ParseError) -> Self {
        BError::from(err).into()
    }
}

impl From<ofdb_entities::nonce::EmailNonceDecodingError> for AppError {
    fn from(err: ofdb_entities::nonce::EmailNonceDecodingError) -> Self {
        BError::from(err).into()
    }
}

impl From<ofdb_entities::nonce::ReviewNonceDecodingError> for AppError {
    fn from(err: ofdb_entities::nonce::ReviewNonceDecodingError) -> Self {
        BError::from(err).into()
    }
}

#[derive(Debug, Error)]
pub enum BError {
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

impl From<String> for BError {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<ofdb_entities::password::ParseError> for BError {
    fn from(_: ofdb_entities::password::ParseError) -> Self {
        Self::Parameter(ParameterError::Password)
    }
}

impl From<ofdb_entities::event::RegistrationTypeParseError> for BError {
    fn from(_: ofdb_entities::event::RegistrationTypeParseError) -> Self {
        Self::Parameter(ParameterError::RegistrationType)
    }
}

impl From<ofdb_entities::nonce::EmailNonceDecodingError> for BError {
    fn from(_: ofdb_entities::nonce::EmailNonceDecodingError) -> Self {
        Self::Parameter(ParameterError::InvalidNonce)
    }
}

impl From<ofdb_entities::nonce::ReviewNonceDecodingError> for BError {
    fn from(_: ofdb_entities::nonce::ReviewNonceDecodingError) -> Self {
        Self::Parameter(ParameterError::InvalidNonce)
    }
}

impl From<ofdb_entities::url::ParseError> for BError {
    fn from(_: ofdb_entities::url::ParseError) -> Self {
        Self::Parameter(ParameterError::Url)
    }
}
