use entities::*;
use super::geo::Coordinate;

pub trait FilterByCategoryIds {
    type Id;
    fn filter_by_category_ids(&self, ids: &[Self::Id]) -> Vec<&Entry>;
}

pub trait FilterByBoundingBox {
    fn filter_by_bounding_box(&self, bb: &[Coordinate]) -> Vec<&Entry>;
}

pub trait InBBox {
    fn in_bbox(&self, bb: &[Coordinate]) -> bool;
}

impl InBBox for Entry {
    fn in_bbox(&self, bb: &[Coordinate]) -> bool {
        // TODO: either return a Result or create a bounding box struct
        if bb.len() != 2 {
            warn!("invalid bounding box: {:?}", bb);
            return false;
        }
        bb.len() == 2 && self.lat >= bb[0].lat && self.lng >= bb[0].lng && self.lat <= bb[1].lat &&
        self.lng <= bb[1].lng
    }
}

impl FilterByCategoryIds for Vec<Entry> {
    type Id = String;
    fn filter_by_category_ids(&self, ids: &[String]) -> Vec<&Entry> {
        self.iter()
            .filter(|e| ids.iter().any(|c| e.categories.iter().any(|x| x == c)))
            .collect()
    }
}

impl FilterByBoundingBox for Vec<Entry> {
    fn filter_by_bounding_box(&self, bb: &[Coordinate]) -> Vec<&Entry> {
        self.iter()
            .filter(|&e| e.in_bbox(bb))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn new_entry(lat: f64, lng: f64, cats: Option<Vec<String>>) -> Entry {
        Entry{
            id          : "foo".into(),
            created     : 0,
            version     : 0,
            title       : "foo".into(),
            description : "bar".into(),
            lat         : lat,
            lng         : lng,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : cats.unwrap_or(vec![]),
            license     : None,
        }
    }

    #[test]
    fn is_in_bounding_box() {
        let bb = vec![Coordinate {
                          lat: -10.0,
                          lng: -10.0,
                      },
                      Coordinate {
                          lat: 10.0,
                          lng: 10.0,
                      }];
        let e = new_entry(5.0, 5.0, None);
        assert_eq!(e.in_bbox(&bb), true);
        let e = new_entry(10.1, 10.0, None);
        assert_eq!(e.in_bbox(&bb), false);
    }

    #[test]
    fn is_in_invalid_bounding_box() {
        let bb = vec![Coordinate {
                          lat: 10.0,
                          lng: 10.0,
                      }];
        let e = new_entry(5.0, 5.0, None);
        assert_eq!(e.in_bbox(&bb), false);
    }

    #[test]
    fn filter_by_bounding_box() {
        let bb = vec![Coordinate {
                          lat: -10.0,
                          lng: -10.0,
                      },
                      Coordinate {
                          lat: 10.0,
                          lng: 10.0,
                      }];
        let entries = vec![new_entry(5.0, 5.0, None),
                           new_entry(-5.0, 5.0, None),
                           new_entry(10.0, 10.1, None)];
        assert_eq!(entries.filter_by_bounding_box(&bb).len(), 2);
    }

    #[test]
    fn filter_by_category() {
        let entries = vec![
            new_entry(5.0, 5.0, Some(vec!["a".into()])),
            new_entry(-5.0, 5.0, Some(vec!["c".into()])),
            new_entry(10.0, 10.1, Some(vec!["b".into(), "a".into()]))];
        assert_eq!(entries.filter_by_category_ids(&["a".into(), "b".into()]).len(),2);
        assert_eq!(entries.filter_by_category_ids(&["b".into()]).len(),1);
        assert_eq!(entries.filter_by_category_ids(&["c".into()]).len(),1);
    }

}
