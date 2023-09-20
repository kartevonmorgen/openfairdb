use gloo_net::http::{Request, RequestBuilder};
use serde::de::DeserializeOwned;
use web_sys::RequestCredentials;

use ofdb_boundary::*;

use crate::{into_json, Result};

/// Authorized OpenFairDB API
#[derive(Clone)]
pub struct UserApi {
    url: &'static str,
    token: JwtToken,
}

impl UserApi {
    pub const fn new(url: &'static str, token: JwtToken) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token.token)
    }
    async fn send<T>(&self, req: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = req
            .header("Authorization", &self.auth_header_value())
            .header("Content-Type", "application/json")
            .send()
            .await?;
        into_json(response).await
    }
    pub async fn user_info(&self) -> Result<User> {
        let url = format!("{}/users/current", self.url);
        let request = Request::get(&url).credentials(RequestCredentials::Include);
        self.send(request).await
    }
    pub async fn bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        let url = format!("{}/bbox-subscriptions", self.url);
        self.send(Request::get(&url)).await
    }
    pub async fn unsubscribe_all_bboxes(&self) -> Result<()> {
        let url = format!("{}/unsubscribe-all-bboxes", self.url);
        self.send(Request::delete(&url)).await
    }
    pub async fn logout(&self) -> Result<()> {
        let url = format!("{}/logout", self.url);
        let request = Request::post(&url).credentials(RequestCredentials::Include);
        self.send(request).await
    }
    pub fn token(&self) -> &JwtToken {
        &self.token
    }
    pub async fn archive_events(&self, ids: &[&str]) -> Result<()> {
        let url = format!("{}/events/{}/archive", self.url, ids.join(","));
        let request = Request::post(&url).credentials(RequestCredentials::Include);
        self.send(request).await
    }
}
