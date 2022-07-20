use std::io;

use diesel::r2d2;
use diesel_migrations::RunMigrationsError;
use thiserror::Error;

use crate::core::error::Error as BError;
use ofdb_core::repositories::Error as RepoError;

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

impl From<RunMigrationsError> for AppError {
    fn from(err: RunMigrationsError) -> AppError {
        AppError::Other(err.into())
    }
}

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum AppError {
    #[error(transparent)]
    Business(#[from] BError),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    //    from(err: io::Error) -> (err.into())
    //}
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    R2d2(#[from] r2d2::PoolError),
    #[error(transparent)]
    CsvIntoInner(#[from] ::csv::IntoInnerError<::csv::Writer<Vec<u8>>>),
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
