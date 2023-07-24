use gloo_net::http::Response;
use serde::de::DeserializeOwned;
use thiserror::Error;

mod clearance;
mod public;
mod user;

pub use self::{clearance::*, public::*, user::*};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Fetch(String),

    #[error("{0:?}")]
    Api(#[from] ofdb_boundary::Error),
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Self::Fetch(format!("{err}"))
    }
}

pub async fn into_json<T>(response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    // ensure we've got 2xx status
    if response.ok() {
        Ok(response.json().await?)
    } else {
        Err(response.json::<ofdb_boundary::Error>().await?.into())
    }
}
