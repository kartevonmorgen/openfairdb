// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use json::Entry;
use geojson::Bbox;

pub trait FilterByCategoryIds {
  type Id;
  fn filter_by_category_ids(&self, ids: &Vec<Self::Id>) -> Vec<Entry>;
}

pub trait FilterByBoundingBox {
  fn filter_by_bounding_box(&self, bb: &Bbox) -> Self;
}

pub trait InBbox {
  fn in_bbox(&self, bb:&Bbox) -> bool;
}

impl InBbox for Entry {
  fn in_bbox(&self, bb:&Bbox) -> bool {
    bb.len() == 4 && self.lat >= bb[0] && self.lng >= bb[1] && self.lat <= bb[2] && self.lng <= bb[3]
  }
}

impl FilterByCategoryIds for Vec<Entry> {
  type Id = String;
  fn filter_by_category_ids(&self, ids:&Vec<String>) -> Vec<Entry> {
    self
     .iter()
     .filter(|&e|
        ids.iter().any(|c|
          e.clone().categories.unwrap_or(vec!()).iter().any(|x| x == c)))
     .map(|x|x.clone())
     .collect()
  }
}

impl FilterByBoundingBox for Vec<Entry> {
  fn filter_by_bounding_box(&self, bb:&Bbox) -> Vec<Entry> {
    self
     .iter()
     .filter(|&e| e.in_bbox(bb))
     .map(|x|x.clone())
     .collect()
  }
}
