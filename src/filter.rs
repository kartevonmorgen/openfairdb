// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use json::Entry;
use geo::Coordinate;

pub trait FilterByCategoryIds {
    type Id;
    fn filter_by_category_ids(&self, ids: &Vec<Self::Id>) -> Vec<Entry>;
}

pub trait FilterByBoundingBox {
    fn filter_by_bounding_box(&self, bb: &Vec<Coordinate>) -> Self;
}

pub trait InBbox {
    fn in_bbox(&self, bb: &Vec<Coordinate>) -> bool;
}

impl InBbox for Entry {
    fn in_bbox(&self, bb: &Vec<Coordinate>) -> bool {
        bb.len() == 2 && self.lat >= bb[0].lat && self.lng >= bb[0].lng && self.lat <= bb[1].lat &&
        self.lng <= bb[1].lng
    }
}

impl FilterByCategoryIds for Vec<Entry> {
    type Id = String;
    fn filter_by_category_ids(&self, ids: &Vec<String>) -> Vec<Entry> {
        self.iter()
            .filter(|&e| {
                ids.iter().any(|c| e.clone().categories.unwrap_or(vec![]).iter().any(|x| x == c))
            })
            .map(|x| x.clone())
            .collect()
    }
}

impl FilterByBoundingBox for Vec<Entry> {
    fn filter_by_bounding_box(&self, bb: &Vec<Coordinate>) -> Vec<Entry> {
        self.iter()
            .filter(|&e| e.in_bbox(bb))
            .map(|x| x.clone())
            .collect()
    }
}
