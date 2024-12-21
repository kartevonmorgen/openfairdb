use gloo_net::http::Request;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use web_sys::RequestCredentials;

use ofdb_boundary::{
    Credentials, Entry, Event, MapBbox, NewPlace, NewPlaceRating, Rating, RequestPasswordReset,
    SearchResponse, TagFrequency, UpdatePlace,
};

use crate::{bbox_string, into_json, Result, UserApi};

/// Public OpenFairDB API
#[derive(Clone)]
pub struct PublicApi {
    url: String,
}

#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    pub bbox: Option<MapBbox>,
    pub limit: Option<u64>,
    pub tags: Vec<String>,
    pub start_min: Option<i64>,
    pub start_max: Option<i64>,
    pub end_min: Option<i64>,
    pub end_max: Option<i64>,
    pub text: Option<String>,
    pub created_by: Option<String>,
}

impl EventQuery {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        let Self {
            bbox,
            limit,
            tags,
            start_min,
            start_max,
            end_min,
            end_max,
            text,
            created_by,
        } = self;
        bbox.is_none()
            && limit.is_none()
            && tags.is_empty()
            && start_min.is_none()
            && start_max.is_none()
            && end_min.is_none()
            && end_max.is_none()
            && text.is_none()
            && created_by.is_none()
    }
}

impl PublicApi {
    #[must_use]
    pub const fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn search(&self, text: &str, bbox: &MapBbox) -> Result<SearchResponse> {
        let encoded_txt = utf8_percent_encode(text, NON_ALPHANUMERIC);
        let bbox_string = bbox_string(bbox);
        let url = format!("{}/search?text={encoded_txt}&bbox={bbox_string}", self.url);
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
        Ok(UserApi::new(self.url.clone(), token))
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

    pub async fn events(&self, query: &EventQuery) -> Result<Vec<Event>> {
        let mut url = format!("{}/events", self.url);
        if !query.is_empty() {
            let EventQuery {
                bbox,
                limit,
                tags,
                start_min,
                start_max,
                end_min,
                end_max,
                text,
                created_by,
            } = query;
            let mut params = vec![];

            if let Some(bbox) = bbox {
                let bbox_string = bbox_string(bbox);
                params.push(("bbox", bbox_string));
            }
            if let Some(limit) = limit {
                params.push(("limit", limit.to_string()));
            }
            if !tags.is_empty() {
                params.push(("tag", tags.join(",")));
            }
            if let Some(start_min) = start_min {
                params.push(("start_min", start_min.to_string()));
            }
            if let Some(start_max) = start_max {
                params.push(("start_max", start_max.to_string()));
            }
            if let Some(end_min) = end_min {
                params.push(("end_min", end_min.to_string()));
            }
            if let Some(end_max) = end_max {
                params.push(("end_max", end_max.to_string()));
            }
            if let Some(text) = text {
                let encoded_text = utf8_percent_encode(text, NON_ALPHANUMERIC);
                params.push(("text", encoded_text.to_string()));
            }
            if let Some(email) = created_by {
                let encoded_email = utf8_percent_encode(email.as_str(), NON_ALPHANUMERIC);
                params.push(("created_by", encoded_email.to_string()));
            }
            let params = params
                .into_iter()
                .map(|(key, value)| [key, &value].join("="))
                .collect::<Vec<_>>()
                .join("&");
            url = format!("{url}?{params}");
        }
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }

    pub async fn event(&self, id: &str) -> Result<Event> {
        let url = format!("{}/events/{id}", self.url);
        let request = Request::get(&url);
        let response = request.send().await?;
        into_json(response).await
    }

    pub async fn create_place(&self, place: &NewPlace) -> Result<String> {
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

    pub async fn update_place(&self, id: &str, place: &UpdatePlace) -> Result<String> {
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
            url = format!("{url}?");
            if let Some(cnt) = min_count {
                url = format!("{url}&min_count={cnt}");
            }
            if let Some(cnt) = max_count {
                url = format!("{url}&max_count={cnt}");
            }
            if let Some(l) = limit {
                url = format!("{url}&limit={l}");
            }
            if let Some(o) = offset {
                url = format!("{url}&offset={o}");
            }
        }
        let request = Request::get(&url);
        let response = request.send().await?;
        into_json(response).await
    }

    pub async fn ratings(&self, ids: &[&str]) -> Result<Vec<Rating>> {
        let url = format!("{}/ratings/{}", self.url, ids.join(","));
        let response = Request::get(&url).send().await?;
        into_json(response).await
    }

    pub async fn create_place_rating(&self, rating: &NewPlaceRating) -> Result<()> {
        let url = format!("{}/ratings", self.url);
        let response = Request::post(&url).json(rating)?.send().await?;
        into_json(response).await
    }
}
