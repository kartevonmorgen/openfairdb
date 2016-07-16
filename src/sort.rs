// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use json::Entry;
use geo::{self, Coordinate};

trait DistanceTo {
    fn distance_to(&self, &Coordinate) -> f64;
}

impl DistanceTo for Entry {
    fn distance_to(&self, c: &Coordinate) -> f64 {
        geo::distance(&Coordinate {
                          lat: self.lat,
                          lng: self.lng,
                      },
                      c)
    }
}

pub trait SortByDistanceTo {
    fn sort_by_distance_to(&mut self, &Coordinate);
}

impl SortByDistanceTo for Vec<Entry> {
    fn sort_by_distance_to(&mut self, c: &Coordinate) {
        self.sort_by(|a, b| {
            a.distance_to(c)
                .partial_cmp(&b.distance_to(c))
                .unwrap()
        });
    }
}
