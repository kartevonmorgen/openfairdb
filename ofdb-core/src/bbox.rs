use ofdb_entities::{event::*, geo::*, place::*};

const BBOX_LAT_DEG_EXT: f64 = 0.02;
const BBOX_LNG_DEG_EXT: f64 = 0.04;

pub fn extend_bbox(bbox: &MapBbox) -> MapBbox {
    let south_west_lat_deg = LatCoord::min()
        .to_deg()
        .max(bbox.southwest().lat().to_deg() - BBOX_LAT_DEG_EXT);
    let north_east_lat_deg = LatCoord::max()
        .to_deg()
        .min(bbox.northeast().lat().to_deg() + BBOX_LAT_DEG_EXT);
    let mut south_west_lng_deg = bbox.southwest().lng().to_deg() - BBOX_LNG_DEG_EXT;
    if south_west_lng_deg < LngCoord::min().to_deg() {
        // wrap around
        south_west_lng_deg += LngCoord::max().to_deg() - LngCoord::min().to_deg();
    }
    let mut north_east_lng_deg = bbox.northeast().lng().to_deg() + BBOX_LNG_DEG_EXT;
    if north_east_lng_deg > LngCoord::max().to_deg() {
        // wrap around
        north_east_lng_deg -= LngCoord::max().to_deg() - LngCoord::min().to_deg();
    }
    if bbox.southwest().lng() <= bbox.northeast().lng() {
        if south_west_lng_deg > north_east_lng_deg {
            // overflow after wrap around (boundaries switched) -> maximize
            south_west_lng_deg = LngCoord::min().to_deg();
            north_east_lng_deg = LngCoord::max().to_deg();
        }
    } else if south_west_lng_deg < north_east_lng_deg {
        // overflow after wrap around (boundaries switched) -> maximize
        south_west_lng_deg = LngCoord::min().to_deg();
        north_east_lng_deg = LngCoord::max().to_deg();
    }
    let extended_bbox = MapBbox::new(
        MapPoint::from_lat_lng_deg(south_west_lat_deg, south_west_lng_deg),
        MapPoint::from_lat_lng_deg(north_east_lat_deg, north_east_lng_deg),
    );
    debug_assert!(extended_bbox.is_valid());
    extended_bbox
}

pub trait InBBox {
    fn in_bbox(&self, bbox: &MapBbox) -> bool;
}

impl InBBox for Place {
    fn in_bbox(&self, bbox: &MapBbox) -> bool {
        bbox.contains_point(self.location.pos)
    }
}

impl InBBox for Event {
    fn in_bbox(&self, bbox: &MapBbox) -> bool {
        if let Some(ref location) = self.location {
            bbox.contains_point(location.pos)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {

    use ofdb_entities::builders::*;

    use super::*;

    #[test]
    fn is_in_bounding_box() {
        let bb = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, -10.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );
        let e = Place::build()
            .title("foo")
            .description("bar")
            .pos(MapPoint::from_lat_lng_deg(5.0, 5.0))
            .finish();
        assert!(e.in_bbox(&bb));
        let e = Place::build()
            .title("foo")
            .description("bar")
            .pos(MapPoint::from_lat_lng_deg(10.1, 10.0))
            .finish();
        assert!(!e.in_bbox(&bb));
    }

    #[test]
    fn filter_by_bounding_box() {
        let bb = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, -10.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );
        let entries = vec![
            Place::build()
                .pos(MapPoint::from_lat_lng_deg(5.0, 5.0))
                .finish(),
            Place::build()
                .pos(MapPoint::from_lat_lng_deg(-5.0, 5.0))
                .finish(),
            Place::build()
                .pos(MapPoint::from_lat_lng_deg(10.0, 10.1))
                .finish(),
        ];
        assert_eq!(entries.iter().filter(|&x| x.in_bbox(&bb)).count(), 2);
    }

    #[test]
    fn extend_max_bbox() {
        let bbox = MapBbox::new(
            MapPoint::from_lat_lng_deg(-89.99, -179.97),
            MapPoint::from_lat_lng_deg(89.99, 179.97),
        );
        let ext_bbox = extend_bbox(&bbox);
        assert!(ext_bbox.is_valid());
        assert_eq!(ext_bbox.southwest().lat(), LatCoord::min());
        assert_eq!(ext_bbox.northeast().lat(), LatCoord::max());
        assert_eq!(ext_bbox.southwest().lng(), LngCoord::min());
        assert_eq!(ext_bbox.northeast().lng(), LngCoord::max());
    }
}
