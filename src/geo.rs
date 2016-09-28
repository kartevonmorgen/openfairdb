// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use error::{AppError, ParameterError};

// The Earth's radius in kilometers.
static EARTH_RADIUS: f64 = 6371.0;

pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

// distance in km
pub fn distance(a: &Coordinate, b: &Coordinate) -> f64 {
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlng = (b.lng - a.lng).to_radians();

    let a = (dlat / 2.0).sin() * (dlat / 2.0).sin() +
            lat1.cos() * lat2.cos() * (dlng / 2.0).sin() * (dlng / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}

pub fn center(south_west: &Coordinate, north_east: &Coordinate) -> Coordinate {
    Coordinate {
        lat: (south_west.lat + north_east.lat) / 2.0,
        lng: (south_west.lng + north_east.lng) / 2.0,
    }
}

pub fn extract_bbox(s: &str) -> Result<Vec<Coordinate>, AppError> {
    let c = s.split(',')
        .map(|x| x.parse::<f64>())
        .filter_map(|x| x.ok())
        .collect::<Vec<f64>>();

    match c.len() {
        4 => {
            Ok(vec![Coordinate {
                        lat: c[0],
                        lng: c[1],
                    },
                    Coordinate {
                        lat: c[2],
                        lng: c[3],
                    }])
        }
        _ => Err(ParameterError::Bbox).map_err(AppError::Parameter),
    }
}

#[test]
fn extract_bbox_test() {
    assert!(extract_bbox("5,4,3").is_err());
}
