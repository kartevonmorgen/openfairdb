// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use json::Entry;
use geo::Coordinate;

pub trait FilterByCategoryIds {
    type Id;
    fn filter_by_category_ids(&self, ids: &[Self::Id]) -> Vec<Entry>;
}

pub trait FilterByBoundingBox {
    fn filter_by_bounding_box(&self, bb: &[Coordinate]) -> Self;
}

pub trait InBbox {
    fn in_bbox(&self, bb: &[Coordinate]) -> bool;
}

impl InBbox for Entry {
    fn in_bbox(&self, bb: &[Coordinate]) -> bool {
        bb.len() == 2 && self.lat >= bb[0].lat && self.lng >= bb[0].lng && self.lat <= bb[1].lat &&
        self.lng <= bb[1].lng
    }
}

impl FilterByCategoryIds for Vec<Entry> {
    type Id = String;
    fn filter_by_category_ids(&self, ids: &[String]) -> Vec<Entry> {
        self.iter()
            .filter(|&e| {
                ids.iter().any(|c| e.clone().categories.unwrap_or_else(||vec![]).iter().any(|x| x == c))
            })
            .cloned()
            .collect()
    }
}

impl FilterByBoundingBox for Vec<Entry> {
    fn filter_by_bounding_box(&self, bb: &[Coordinate]) -> Vec<Entry> {
        self.iter()
            .filter(|&e| e.in_bbox(bb))
            .cloned()
            .collect()
    }
}
