use gloo_net::http::Request;
use ofdb_boundary::*;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use web_sys::RequestCredentials;

use crate::{into_json, Result, UserApi};

/// Public OpenFairDB API
#[derive(Clone, Copy)]
pub struct PublicApi {
    url: &'static str,
}

impl PublicApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }
    pub async fn search(&self, text: &str, bbox: &MapBbox) -> Result<SearchResponse> {
        let encoded_txt = utf8_percent_encode(text, NON_ALPHANUMERIC);
        let MapBbox { sw, ne } = bbox;
        let bbox_str = format!("{},{},{},{}", sw.lat, sw.lng, ne.lat, ne.lng);
        let url = format!("{}/search?text={encoded_txt}&bbox={bbox_str}", self.url);
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn register(&self, credentials: &Credentials) -> Result<()> {
        let url = format!("{}/users", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        into_json(response).await
    }
    pub async fn login(&self, credentials: &Credentials) -> Result<UserApi> {
        let url = format!("{}/login", self.url);
        let response = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .json(credentials)?
            .send()
            .await?;
        let token = into_json(response).await?;
        Ok(UserApi::new(self.url, token))
    }
    pub async fn request_password_reset(&self, email: String) -> Result<()> {
        let url = format!("{}/users/reset-password-request", self.url);
        let response = Request::post(&url)
            .json(&RequestPasswordReset { email })?
            .send()
            .await?;
        into_json(response).await
    }
    pub async fn entries(&self, ids: &[&str]) -> Result<Vec<Entry>> {
        let url = format!("{}/entries/{}", self.url, ids.join(","));
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn create_place(&self, place: &NewPlace) -> Result<()> {
        let url = format!("{}/entries", self.url);
        let request = Request::post(&url).json(place)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn tags(&self) -> Result<Vec<String>> {
        let url = format!("{}/tags", self.url);
        let request = Request::get(&url);
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn count_entries(&self) -> Result<usize> {
        let url = format!("{}/count/entries", self.url);
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn count_tags(&self) -> Result<usize> {
        let url = format!("{}/count/tags", self.url);
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn update_place(&self, id: &str, place: &UpdatePlace) -> Result<()> {
        let url = format!("{}/entries/{}", self.url, id);
        let request = Request::put(&url).json(place)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn most_popular_tags(
        &self,
        min_count: Option<usize>,
        max_count: Option<usize>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<TagFrequency>> {
        let mut url = format!("{}/entries/most-popular-tags", self.url);
        if min_count.or(max_count).or(limit).or(offset).is_some() {
            url = format!("{}?", url);
            if let Some(cnt) = min_count {
                url = format!("{}&min_count={}", url, cnt);
            }
            if let Some(cnt) = max_count {
                url = format!("{}&max_count={}", url, cnt);
            }
            if let Some(l) = limit {
                url = format!("{}&limit={}", url, l);
            }
            if let Some(o) = offset {
                url = format!("{}&offset={}", url, o);
            }
        }
        let request = Request::get(&url);
        let response = request.send().await?;
        into_json(response).await
    }
}
