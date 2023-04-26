use async_trait::async_trait;
use gloo_net::http::{Request, Response};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::de::DeserializeOwned;
use thiserror::Error;

use ofdb_boundary::*;

#[async_trait(?Send)]
pub trait PublicApi {
    async fn search(&self, text: &str, bbox: &MapBbox) -> Result<SearchResponse>;
    async fn count_entries(&self) -> Result<usize>;
    async fn count_tags(&self) -> Result<usize>;
}

#[async_trait(?Send)]
pub trait UserApi: PublicApi {
    async fn user_info(&self) -> Result<User>;
    async fn bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;
    async fn unsubscribe_all_bboxes(&self) -> Result<()>;
    async fn logout(&self) -> Result<()>;
    fn token(&self) -> &JwtToken;
}

/// Public OpenFairDB API
#[derive(Clone, Copy)]
pub struct UnauthorizedApi {
    url: &'static str,
}

#[derive(Clone)]
pub struct AuthorizedApi {
    url: &'static str,
    token: JwtToken,
}

impl UnauthorizedApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }
    pub async fn register(&self, credentials: &Credentials) -> Result<()> {
        let url = format!("{}/users", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        into_json(response).await
    }
    pub async fn login(&self, credentials: &Credentials) -> Result<AuthorizedApi> {
        let url = format!("{}/login", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        let token = into_json(response).await?;
        Ok(AuthorizedApi::new(self.url, token))
    }
    pub async fn request_password_reset(&self, email: String) -> Result<()> {
        let url = format!("{}/users/reset-password-request", self.url);
        let response = Request::post(&url)
            .json(&RequestPasswordReset { email })?
            .send()
            .await?;
        into_json(response).await
    }
}

impl AuthorizedApi {
    pub const fn new(url: &'static str, token: JwtToken) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token.token)
    }
    async fn send<T>(&self, req: Request) -> Result<T>
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
        self.send(Request::get(&url)).await
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
        self.send(Request::post(&url)).await
    }
    pub fn token(&self) -> &JwtToken {
        &self.token
    }
}

#[async_trait(?Send)]
impl PublicApi for UnauthorizedApi {
    async fn search(&self, text: &str, bbox: &MapBbox) -> Result<SearchResponse> {
        search(self.url, text, bbox).await
    }
    async fn count_entries(&self) -> Result<usize> {
        count_entries(self.url).await
    }
    async fn count_tags(&self) -> Result<usize> {
        count_tags(self.url).await
    }
}

#[async_trait(?Send)]
impl PublicApi for AuthorizedApi {
    async fn search(&self, text: &str, bbox: &MapBbox) -> Result<SearchResponse> {
        search(self.url, text, bbox).await
    }
    async fn count_entries(&self) -> Result<usize> {
        count_entries(self.url).await
    }
    async fn count_tags(&self) -> Result<usize> {
        count_tags(self.url).await
    }
}

#[async_trait(?Send)]
impl UserApi for AuthorizedApi {
    async fn user_info(&self) -> Result<User> {
        self.user_info().await
    }
    async fn bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        self.bbox_subscriptions().await
    }
    async fn unsubscribe_all_bboxes(&self) -> Result<()> {
        self.unsubscribe_all_bboxes().await
    }
    async fn logout(&self) -> Result<()> {
        self.logout().await
    }
    fn token(&self) -> &JwtToken {
        self.token()
    }
}

async fn search(endpoint_url: &str, text: &str, bbox: &MapBbox) -> Result<SearchResponse> {
    let encoded_txt = utf8_percent_encode(text, NON_ALPHANUMERIC);
    let MapBbox { sw, ne } = bbox;
    let bbox_str = format!("{},{},{},{}", sw.lat, sw.lng, ne.lat, ne.lng);
    let url = format!("{endpoint_url}/search?text={encoded_txt}&bbox={bbox_str}");
    let response = Request::get(&url).send().await?;
    into_json(response).await
}

async fn count_entries(endpoint_url: &str) -> Result<usize> {
    let url = format!("{endpoint_url}/count/entries");
    let response = Request::get(&url).send().await?;
    into_json(response).await
}

async fn count_tags(endpoint_url: &str) -> Result<usize> {
    let url = format!("{endpoint_url}/count/tags");
    let response = Request::get(&url).send().await?;
    into_json(response).await
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Fetch(String),
    #[error("{0:?}")]
    Api(ofdb_boundary::Error),
}

// TODO: use thiserror in ofdb_boundary::Error
impl From<ofdb_boundary::Error> for Error {
    fn from(e: ofdb_boundary::Error) -> Self {
        Self::Api(e)
    }
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Self::Fetch(format!("{err}"))
    }
}

async fn into_json<T>(response: Response) -> Result<T>
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
