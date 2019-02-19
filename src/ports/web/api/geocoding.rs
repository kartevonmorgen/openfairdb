use crate::core::entities::Address;

use ::geocoding::{Forward, Opencage};
use itertools::Itertools;
use std::env;

lazy_static! {
    static ref OC_API_KEY: Option<String> = match env::var("OPENCAGE_API_KEY") {
        Ok(key) => Some(key),
        Err(_) => {
            warn!("No OpenCage API key found");
            None
        }
    };
}

fn address_to_forward_query_string(addr: &Address) -> String {
    let addr_parts = [&addr.street, &addr.zip, &addr.city, &addr.country];
    addr_parts.into_iter().filter_map(|x| x.as_ref()).join(",")
}

fn oc_resolve_address_lat_lng(oc_api_key: String, addr: &Address) -> Option<(f64, f64)> {
    let oc_req = Opencage::new(oc_api_key);
    let addr_str = address_to_forward_query_string(addr);
    match oc_req.forward(&addr_str) {
        Ok(res) => {
            if !res.is_empty() {
                let point = &res[0];
                debug!("Resolved address location '{}': {:?}", addr_str, point);
                return Some((point.lat(), point.lng()));
            }
        }
        Err(err) => {
            warn!("Failed to resolve address location '{}': {}", addr_str, err);
        }
    }
    None
}

//TODO: use a trait for the geocoding API and test it with a mock
pub fn resolve_address_lat_lng(addr: &Address) -> Option<(f64, f64)> {
    if addr.is_empty() {
        None
    } else {
        OC_API_KEY
            .as_ref()
            .and_then(|key| oc_resolve_address_lat_lng(key.clone(), addr))
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
