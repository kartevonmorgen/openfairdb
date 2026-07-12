use itertools::Itertools;
use serde::Deserialize;

use ofdb_core::gateways::geocode::GeoCodingGateway;
use ofdb_entities::address::Address;

const OPENCAGE_FORWARD_URL: &str = "https://api.opencagedata.com/geocode/v1/json";

pub struct OpenCage {
    api_key: Option<String>,
}

impl OpenCage {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

fn address_to_forward_query_string(addr: &Address) -> String {
    let addr_parts = [&addr.street, &addr.zip, &addr.city, &addr.country];
    addr_parts.iter().filter_map(|x| x.as_ref()).join(",")
}

// Minimal subset of the OpenCage forward geocoding response.
// See https://opencagedata.com/api#forward-resp
#[derive(Debug, Deserialize)]
struct OpencageResponse {
    results: Vec<OpencageResult>,
}

#[derive(Debug, Deserialize)]
struct OpencageResult {
    geometry: OpencageGeometry,
}

#[derive(Debug, Deserialize)]
struct OpencageGeometry {
    lat: f64,
    lng: f64,
}

fn oc_resolve_address_lat_lng(oc_api_key: &str, addr: &Address) -> Option<(f64, f64)> {
    let addr_str = address_to_forward_query_string(addr);
    match request_forward_geocoding(oc_api_key, &addr_str) {
        Ok(res) => {
            if let Some(geometry) = res.results.into_iter().next().map(|r| r.geometry) {
                log::debug!("Resolved address location '{addr_str}': {geometry:?}");
                return Some((geometry.lat, geometry.lng));
            }
        }
        Err(err) => {
            log::warn!("Failed to resolve address location '{addr_str}': {err}");
        }
    }
    None
}

fn request_forward_geocoding(
    oc_api_key: &str,
    query: &str,
) -> Result<OpencageResponse, reqwest::Error> {
    reqwest::blocking::Client::new()
        .get(OPENCAGE_FORWARD_URL)
        .query(&[("q", query), ("key", oc_api_key)])
        .send()?
        .error_for_status()?
        .json()
}

impl GeoCodingGateway for OpenCage {
    fn resolve_address_lat_lng(&self, addr: &Address) -> Option<(f64, f64)> {
        if addr.is_empty() {
            None
        } else {
            self.api_key
                .as_ref()
                .and_then(|key| oc_resolve_address_lat_lng(key, addr))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_to_forward_query_string_partial() {
        let mut addr = Address {
            street: Some("A street".into()),
            city: Some("A city".into()),
            ..Default::default()
        };
        assert_eq!("A street,A city", address_to_forward_query_string(&addr));
        addr.country = Some("A country".into());
        assert_eq!(
            "A street,A city,A country",
            address_to_forward_query_string(&addr)
        );
        addr.street = None;
        addr.zip = Some("1234".into());
        assert_eq!(
            "1234,A city,A country",
            address_to_forward_query_string(&addr)
        );
    }
}
