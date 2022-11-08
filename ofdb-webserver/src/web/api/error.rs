use super::json_error_response;
use anyhow::anyhow;
use ofdb_application::error::{AppError, BError};
pub use ofdb_core::{repositories::Error as RepoError, usecases::Error as ParameterError};
use rocket::{
    self,
    http::Status,
    response::{self, Responder},
    serde::json::Error as JsonError,
};
use std::{io, string};
use thiserror::Error;

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

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Self::Other(anyhow!(err))
    }
}

impl<T: io::Write> From<csv::IntoInnerError<csv::Writer<T>>> for Error {
    fn from(err: csv::IntoInnerError<csv::Writer<T>>) -> Self {
        Self::Other(anyhow!("{err}"))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Other(anyhow!(err))
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::Other(anyhow!(err))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &rocket::Request) -> response::Result<'o> {
        match self {
            Error::App(err) => {
                if let AppError::Business(err) = &err {
                    match err {
                        BError::Parameter(ref err) => {
                            return match *err {
                                ParameterError::Credentials | ParameterError::Unauthorized => {
                                    json_error_response(req, err, Status::Unauthorized)
                                }
                                ParameterError::Forbidden
                                | ParameterError::ModeratedTag
                                | ParameterError::EmailNotConfirmed => {
                                    json_error_response(req, err, Status::Forbidden)
                                }
                                _ => json_error_response(req, err, Status::BadRequest),
                            };
                        }
                        BError::Repo(RepoError::NotFound) => {
                            return json_error_response(req, err, Status::NotFound);
                        }
                        _ => {}
                    }
                }
                error!("Error: {err}");
                Err(Status::InternalServerError)
            }
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

impl From<ofdb_core::usecases::Error> for Error {
    fn from(err: ofdb_core::usecases::Error) -> Self {
        Self::App(err.into())
    }
}

impl From<ofdb_entities::email::EmailAddressParseError> for Error {
    fn from(err: ofdb_entities::email::EmailAddressParseError) -> Self {
        Self::OtherWithStatus(err.into(), Status::BadRequest)
    }
}
