use gloo_net::http::Response;
use serde::de::DeserializeOwned;

use crate::Result;

pub fn auth_header_value(token: &str) -> String {
    format!("Bearer {}", token)
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
