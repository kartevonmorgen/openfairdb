use business::error::ParameterError;
use entities::{Coordinate, Bbox};

// The Earth's radius in kilometers.
static EARTH_RADIUS: f64 = 6371.0;

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

pub fn extract_bbox(s: &str) -> Result<Vec<Coordinate>, ParameterError> {
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
        _ => Err(ParameterError::Bbox),
    }
}

pub fn is_in_bbox(lat: &f64, lng: &f64, bbox: &Bbox) -> bool {
    *lat >= bbox.south_west.lat &&
    *lng >= bbox.south_west.lng &&
    *lat <= bbox.north_east.lat &&
    *lng <= bbox.north_east.lng
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn no_distance() {
        let c0 = Coordinate { lat: 0.0, lng: 0.0 };
        assert_eq!(distance(&c0, &c0), 0.0);
        let c10 = Coordinate {
            lat: 10.0,
            lng: 10.0,
        };
        assert_eq!(distance(&c10, &c10), 0.0);
    }

    #[test]
    fn real_distance() {

        // 48° 47′ N, 9° 11′ O
        let stuttgart = Coordinate {
            lat: 48.7755,
            lng: 9.1827,
        };

        // 49° 29′ N, 8° 28′ O
        let mannheim = Coordinate {
            lat: 49.4836,
            lng: 8.4630,
        };

        assert!(distance(&stuttgart, &mannheim) > 92.0);
        assert!(distance(&stuttgart, &mannheim) < 96.0);
    }

    #[test]
    fn symetric_distance() {
        let a = Coordinate {
            lat: 80.0,
            lng: 0.0,
        };
        let b = Coordinate {
            lat: 90.0,
            lng: 20.0,
        };
        assert_eq!(distance(&a, &b), distance(&b, &a));
    }

    use std::f64::{NAN, INFINITY};

    #[test]
    fn distance_with_invalid_coordinates() {
        let a = Coordinate {
            lat: 10.0,
            lng: NAN,
        };
        let b = Coordinate {
            lat: 20.0,
            lng: 20.0,
        };
        assert!(distance(&a, &b).is_nan());
        let a = Coordinate {
            lat: 10.0,
            lng: INFINITY,
        };
        let b = Coordinate {
            lat: 20.0,
            lng: 20.0,
        };
        assert!(distance(&a, &b).is_nan());
    }

    #[test]
    fn extract_bbox_from_str() {
        assert!(extract_bbox("0,-10.0870,200,3.0").is_ok());
        let bb = extract_bbox("0,10,20,30");
        assert!(bb.is_ok());
        let bb = bb.unwrap();
        assert_eq!(bb.len(), 2);
        assert_eq!(bb[0].lat, 0.0);
        assert_eq!(bb[0].lng, 10.0);
        assert_eq!(bb[1].lat, 20.0);
        assert_eq!(bb[1].lng, 30.0);
    }

    #[test]
    fn extract_bbox_from_str_with_missing_lng() {
        assert!(extract_bbox("5,4,3").is_err());
    }

    #[test]
    fn extract_bbox_from_str_with_invalid_chars() {
        assert!(extract_bbox("5,4,3,o").is_err());
        assert!(extract_bbox("5;4;3,0").is_err());
    }

    #[test]
    fn center_of_bounding_box() {
        let res = center(&Coordinate {
                             lat: 50.0,
                             lng: 10.0,
                         },
                         &Coordinate {
                             lat: 60.0,
                             lng: 5.0,
                         });
        assert_eq!(res.lat, 55.0);
        assert_eq!(res.lng, 7.5);

        let res = center(&Coordinate {
                             lat: -20.0,
                             lng: -20.0,
                         },
                         &Coordinate {
                             lat: 10.0,
                             lng: 10.0,
                         });
        assert_eq!(res.lat, -5.0);
        assert_eq!(res.lng, -5.0);

        let res = center(&Coordinate {
                             lat: -10.0,
                             lng: 10.0,
                         },
                         &Coordinate {
                             lat: 10.0,
                             lng: -10.0,
                         });
        assert_eq!(res.lat, 0.0);
        assert_eq!(res.lng, 0.0);
    }

    #[test]
    fn test_is_in_bbox(){
        let bbox1 = Bbox{
            south_west: Coordinate{
                lat: 0.0,
                lng: 0.0
            },
            north_east: Coordinate{
                lat: 10.0,
                lng: 10.0
            }
        };
        let bbox2 = Bbox{
            south_west: Coordinate{
                lat: -10.0,
                lng: 0.0
            },
            north_east: Coordinate{
                lat: 0.0,
                lng: 10.0
            }
        };
        let bbox3 = Bbox{
            south_west: Coordinate{
                lat: -10.0,
                lng: -10.0
            },
            north_east: Coordinate{
                lat: 0.0,
                lng: 0.0
            }
        };
        let bbox4 = Bbox{
            south_west: Coordinate{
                lat: 0.0,
                lng: -10.0
            },
            north_east: Coordinate{
                lat: 10.0,
                lng: 0.0
            }
        };

        let lat1 = 5.0;
        let lng1 = 5.0;
        let lat2 = -5.0;
        let lng2 = 5.0;
        let lat3 = -5.0;
        let lng3 = -5.0;
        let lat4 = 5.0;
        let lng4 = -5.0;

        assert!(is_in_bbox(&lat1, &lng1, &bbox1));
        assert!(!is_in_bbox(&lat1, &lng1, &bbox2));
        assert!(!is_in_bbox(&lat1, &lng1, &bbox3));
        assert!(!is_in_bbox(&lat1, &lng1, &bbox4));
        assert!(!is_in_bbox(&lat2, &lng2, &bbox1));
        assert!(is_in_bbox(&lat2, &lng2, &bbox2));
        assert!(!is_in_bbox(&lat2, &lng2, &bbox3));
        assert!(!is_in_bbox(&lat2, &lng2, &bbox4));
        assert!(!is_in_bbox(&lat3, &lng3, &bbox1));
        assert!(!is_in_bbox(&lat3, &lng3, &bbox2));
        assert!(is_in_bbox(&lat3, &lng3, &bbox3));
        assert!(!is_in_bbox(&lat3, &lng3, &bbox4));
        assert!(!is_in_bbox(&lat4, &lng4, &bbox1));
        assert!(!is_in_bbox(&lat4, &lng4, &bbox2));
        assert!(!is_in_bbox(&lat4, &lng4, &bbox3));
        assert!(is_in_bbox(&lat4, &lng4, &bbox4));
    }
}
