use gloo_net::http::Request;
use ofdb_boundary::*;

use crate::{into_json, Result};

/// OpenFairDB Clearance API
#[derive(Clone)]
pub struct ClearanceApi {
    url: &'static str,
    token: String,
}

impl ClearanceApi {
    pub const fn new(url: &'static str, token: String) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token)
    }
    pub async fn place_clearances(&self) -> Result<Vec<PendingClearanceForPlace>> {
        let url = format!("{}/places/clearance", self.url);
        let request = Request::get(&url)
            .header("Authorization", &self.auth_header_value())
            .header("Content-Type", "application/json");
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn place_history(&self, id: &str) -> Result<PlaceHistory> {
        let url = format!("{}/places/{}/history", self.url, id);
        let request = Request::get(&url)
            .header("Authorization", &self.auth_header_value())
            .header("Content-Type", "application/json");
        let response = request.send().await?;
        into_json(response).await
    }
    pub async fn update_place_clearances(
        &self,
        clearances: Vec<ClearanceForPlace>,
    ) -> Result<ResultCount> {
        let url = format!("{}/places/clearance", self.url);
        let request = Request::post(&url)
            .header("Authorization", &self.auth_header_value())
            .json(&clearances)?;
        let response = request.send().await?;
        into_json(response).await
    }
}
