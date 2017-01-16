use entities::*;
use std::cmp::Ordering;
use super::geo::{self, Coordinate};

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
        self.sort_by(|a, _| if a.lat.is_finite() && a.lng.is_finite() {
            Ordering::Less
        } else {
            warn!("inivalid coordinate: {}/{}", a.lat, a.lng);
            Ordering::Greater
        });
        self.sort_by(|a, b| {
            a.distance_to(c).partial_cmp(&b.distance_to(c)).unwrap_or(Ordering::Equal)
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn new_entry(id: &str, lat: f64, lng: f64) -> Entry { Entry{
            id          : id.into(),
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
            categories  : vec![],
            license     : None,
        }
    }

    #[test]
    fn sort_by_distance() {
        let mut entries = vec![new_entry("a", 1.0, 0.0),
                               new_entry("b", 0.0, 0.0),
                               new_entry("c", 1.0, 1.0),
                               new_entry("d", 0.0, 0.5),
                               new_entry("e", -1.0, -1.0)];
        let x = Coordinate {
            lat: 0.0,
            lng: 0.0,
        };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "d");
        assert_eq!(entries[2].id, "a");
        assert_eq!(entries[3].id, "c");
        assert_eq!(entries[4].id, "e");
    }

    use std::f64::{NAN, INFINITY};

    #[test]
    fn sort_with_invalid_coordinates() {
        let mut entries = vec![new_entry("a", 1.0, NAN),
                               new_entry("b", 1.0, INFINITY),
                               new_entry("c", 2.0, 0.0),
                               new_entry("d", NAN, NAN),
                               new_entry("e", 1.0, 0.0)];
        let x = Coordinate {
            lat: 0.0,
            lng: 0.0,
        };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "e");
        assert_eq!(entries[1].id, "c");

        let mut entries =
            vec![new_entry("a", 2.0, 0.0), new_entry("b", 0.0, 0.0), new_entry("c", 1.0, 0.0)];

        let x = Coordinate {
            lat: NAN,
            lng: 0.0,
        };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[1].id, "b");
        assert_eq!(entries[2].id, "c");
    }

}
