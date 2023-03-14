use gloo_net::http::{Request, RequestCredentials, Response};
use ofdb_boundary::*;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::de::DeserializeOwned;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

/// OpenFairDB API
#[derive(Debug, Clone)]
pub struct Api {
    url: String,
}

impl Api {
    pub fn new(url: String) -> Self {
        Self { url }
    }
    pub async fn search(&self, txt: &str, bbox: &MapBbox) -> Result<SearchResponse> {
        let encoded_txt = utf8_percent_encode(txt, NON_ALPHANUMERIC);
        let MapBbox { sw, ne } = bbox;
        let bbox_str = format!("{},{},{},{}", sw.lat, sw.lng, ne.lat, ne.lng);
        let url = format!("{}/search?text={}&bbox={}", self.url, encoded_txt, bbox_str);
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn places(&self, ids: &[String]) -> Result<Vec<Entry>> {
        let ids = ids.join(",");
        let url = format!("{}/entries/{}", self.url, ids);
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }
    pub async fn create_place(&self, place: &NewPlace) -> Result<()> {
        let url = format!("{}/entries", self.url);
        let request = Request::post(&url).json(place)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn update_place(&self, id: &str, place: &UpdatePlace) -> Result<()> {
        let url = format!("{}/entries/{}", self.url, id);
        let request = Request::put(&url).json(place)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn get_places_clearance_with_api_token(
        &self,
        api_token: &str,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        let url = format!("{}/places/clearance", self.url);
        let request = Request::get(&url)
            .header("Authorization", &auth_header_value(api_token))
            .header("Content-Type", "application/json");
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn get_place_history_with_api_token(
        &self,
        api_token: &str,
        id: &str,
    ) -> Result<PlaceHistory> {
        let url = format!("{}/places/{}/history", self.url, id);
        let request = Request::get(&url)
            .header("Authorization", &auth_header_value(api_token))
            .header("Content-Type", "application/json");
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn post_places_clearance_with_api_token(
        &self,
        api_token: &str,
        clearances: Vec<ClearanceForPlace>,
    ) -> Result<ResultCount> {
        let url = format!("{}/places/clearance", self.url);
        let request = Request::post(&url)
            .header("Authorization", &auth_header_value(api_token))
            .json(&clearances)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn post_login(&self, req: &Credentials) -> Result<()> {
        let url = format!("{}/login", self.url);
        let request = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .json(&req)?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn post_logout(&self) -> Result<()> {
        let url = format!("{}/logout", self.url);
        let request = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .json(&())?;
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn get_users_current(&self) -> Result<User> {
        let url = format!("{}/users/current", self.url);
        let request = Request::get(&url).credentials(RequestCredentials::Include);
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn get_tags(&self) -> Result<Vec<String>> {
        let url = format!("{}/tags", self.url);
        let request = Request::get(&url);
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn get_most_popular_tags(
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

fn auth_header_value(token: &str) -> String {
    format!("Bearer {}", token)
}

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
