use crate::core::error::{Error as BError, RepoError};
use crate::infrastructure::error::AppError;
use anyhow::anyhow;
use rocket::{
    self,
    http::Status,
    response::{Responder, Response},
};
use rocket_contrib::json::JsonError;
use thiserror::Error;

use super::json_error_response;

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    #[error(transparent)]
    App(#[from] AppError),
    #[error("{0}")]
    OtherWithStatus(#[source] anyhow::Error, Status),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<JsonError<'_>> for Error {
    fn from(err: JsonError) -> Self {
        match err {
            JsonError::Io(err) => Self::OtherWithStatus(anyhow!(err), Status::UnprocessableEntity),
            JsonError::Parse(_str, err) => {
                Self::OtherWithStatus(anyhow!(err), Status::UnprocessableEntity)
            }
        }
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, req: &rocket::Request) -> std::result::Result<Response<'r>, Status> {
        match self {
            Error::App(err) => err.respond_to(req),
            Error::OtherWithStatus(err, status) => json_error_response(req, &err, status),
            Error::Other(err) => json_error_response(req, &err, Status::ImATeapot),
        }
    }
}

impl From<RepoError> for Error {
    fn from(err: RepoError) -> Self {
        AppError::from(err).into()
    }
}

impl From<BError> for Error {
    fn from(err: BError) -> Self {
        AppError::from(err).into()
    }
}

impl From<ofdb_entities::password::ParseError> for Error {
    fn from(err: ofdb_entities::password::ParseError) -> Self {
        AppError::from(err).into()
    }
}

impl From<ofdb_entities::nonce::EmailNonceDecodingError> for Error {
    fn from(err: ofdb_entities::nonce::EmailNonceDecodingError) -> Self {
        AppError::from(err).into()
    }
}
