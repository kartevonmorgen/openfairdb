use crate::{
    core::{prelude::*, usecases},
    infrastructure::cfg::Cfg,
};

mod flows {
    pub use super::super::flows::{prelude::*, tests::prelude::BackendFixture, Result};
}

mod clearance;
mod search;

pub fn default_new_place() -> usecases::NewPlace {
    usecases::NewPlace {
        title: Default::default(),
        description: Default::default(),
        categories: Default::default(),
        contact_name: None,
        email: None,
        telephone: None,
        lat: Default::default(),
        lng: Default::default(),
        street: None,
        zip: None,
        city: None,
        country: None,
        state: None,
        tags: vec![],
        homepage: None,
        opening_hours: None,
        founded_on: None,
        license: "CC0-1.0".into(),
        image_url: None,
        image_link_url: None,
        custom_links: vec![],
    }
}

fn default_search_request<'a>() -> usecases::SearchRequest<'a> {
    usecases::SearchRequest {
        bbox: MapBbox::new(
            MapPoint::from_lat_lng_deg(-90, -180),
            MapPoint::from_lat_lng_deg(90, 180),
        ),
        org_tag: None,
        categories: vec![],
        hash_tags: vec![],
        ids: vec![],
        status: vec![],
        text: None,
    }
}
