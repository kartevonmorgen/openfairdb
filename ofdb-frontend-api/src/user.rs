use gloo_net::http::{Request, RequestBuilder};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{de::DeserializeOwned, Serialize};
use web_sys::RequestCredentials;

use ofdb_boundary::{BboxSubscription, JwtToken, MapBbox, Review, User};

use crate::{bbox_string, into_json, Result};

/// Authorized OpenFairDB API
#[derive(Clone)]
pub struct UserApi {
    url: &'static str,
    token: JwtToken,
}

impl UserApi {
    #[must_use]
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
        let response = self
            .add_auth_headers(req)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        into_json(response).await
    }
    async fn send_json<D, T>(&self, req: RequestBuilder, data: &D) -> Result<T>
    where
        T: DeserializeOwned,
        D: Serialize,
    {
        let response = self.add_auth_headers(req).json(data)?.send().await?;
        into_json(response).await
    }
    fn add_auth_headers(&self, req: RequestBuilder) -> RequestBuilder {
        req.header("Authorization", &self.auth_header_value())
            .credentials(RequestCredentials::Include)
    }
    pub async fn user_info(&self) -> Result<User> {
        let url = format!("{}/users/current", self.url);
        let request = Request::get(&url);
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
        let request = Request::post(&url);
        self.send(request).await
    }
    #[must_use]
    pub fn token(&self) -> &JwtToken {
        &self.token
    }
    pub async fn archive_events(&self, ids: &[&str]) -> Result<()> {
        let url = format!("{}/events/{}/archive", self.url, ids.join(","));
        let request = Request::post(&url);
        self.send(request).await
    }
    pub async fn review_places(&self, ids: &[&str], review: &Review) -> Result<()> {
        let url = format!("{}/places/{}/review", self.url, ids.join(","));
        let request = Request::post(&url);
        self.send_json(request, review).await
    }

    // TODO: add other options like `limit`, `tags`, etc.
    pub async fn export_csv(&self, bbox: &MapBbox, search_term: Option<&str>) -> Result<String> {
        let bbox_string = bbox_string(bbox);
        let search = search_term
            .map(|term| {
                let encoded_txt = utf8_percent_encode(term, NON_ALPHANUMERIC);
                format!("&text={encoded_txt}")
            })
            .unwrap_or_default();
        let url = format!("{}/export/entries.csv?bbox={bbox_string}{search}", self.url);
        let request = self.add_auth_headers(Request::get(&url));
        let response = request.send().await?;
        let csv_string = response.text().await?;
        Ok(csv_string)
    }
}
