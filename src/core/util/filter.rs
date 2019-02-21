use super::super::entities::*;
use super::geo::*;

const BBOX_LAT_DEG_EXT: f64 = 0.02;
const BBOX_LNG_DEG_EXT: f64 = 0.04;

pub fn extend_bbox(bbox: &MapBbox) -> MapBbox {
    let south_west_lat_deg = LatCoord::min()
        .to_deg()
        .max(bbox.south_west().lat().to_deg() - BBOX_LAT_DEG_EXT);
    let north_east_lat_deg = LatCoord::max()
        .to_deg()
        .min(bbox.north_east().lat().to_deg() + BBOX_LAT_DEG_EXT);
    let mut south_west_lng_deg = bbox.south_west().lng().to_deg() - BBOX_LNG_DEG_EXT;
    if south_west_lng_deg < LngCoord::min().to_deg() {
        // wrap around
        south_west_lng_deg += LngCoord::max().to_deg() - LngCoord::min().to_deg();
    }
    let mut north_east_lng_deg = bbox.north_east().lng().to_deg() + BBOX_LNG_DEG_EXT;
    if north_east_lng_deg > LngCoord::max().to_deg() {
        // wrap around
        north_east_lng_deg -= LngCoord::max().to_deg() - LngCoord::min().to_deg();
    }
    if bbox.south_west().lng() <= bbox.north_east().lng() {
        if south_west_lng_deg > north_east_lng_deg {
            // overflow after wrap around (boundaries switched) -> maximize
            south_west_lng_deg = LngCoord::min().to_deg();
            north_east_lng_deg = LngCoord::max().to_deg();
        }
    } else {
        if south_west_lng_deg < north_east_lng_deg {
            // overflow after wrap around (boundaries switched) -> maximize
            south_west_lng_deg = LngCoord::min().to_deg();
            north_east_lng_deg = LngCoord::max().to_deg();
        }
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

impl InBBox for Entry {
    fn in_bbox(&self, bbox: &MapBbox) -> bool {
        bbox.contains_point(&MapPoint::from_lat_lng_deg(
            self.location.lat,
            self.location.lng,
        ))
    }
}

impl InBBox for Event {
    fn in_bbox(&self, bbox: &MapBbox) -> bool {
        if let Some(ref location) = self.location {
            bbox.contains_point(&MapPoint::from_lat_lng_deg(location.lat, location.lng))
        } else {
            false
        }
    }
}

#[cfg(feature = "test")]
pub fn entries_by_category_ids<'a>(ids: &'a [String]) -> impl Fn(&Entry) -> bool + 'a {
    move |e| ids.iter().any(|c| e.categories.iter().any(|x| x == c))
}

#[cfg(feature = "test")]
fn entries_by_tags_and_search_text<'a>(
    text: &'a str,
    tags: &'a [String],
) -> impl Fn(&Entry) -> bool + 'a {
    let words = to_words(text);
    move |entry| {
        tags.iter()
            .map(|t| t.to_lowercase())
            .all(|tag| entry.tags.iter().any(|t| *t == tag))
            || ((!text.is_empty()
                && words.iter().any(|word| {
                    entry.title.to_lowercase().contains(word)
                        || entry.description.to_lowercase().contains(word)
                        || entry.tags.iter().any(|tag| tag == word)
                }))
                || (text.is_empty() && tags[0] == ""))
    }
}

#[cfg(feature = "test")]
pub fn entries_by_tags_or_search_text<'a>(
    text: &'a str,
    tags: &'a [String],
) -> Box<Fn(&Entry) -> bool + 'a> {
    if !tags.is_empty() {
        Box::new(entries_by_tags_and_search_text(text, tags))
    } else {
        Box::new(entries_by_search_text(text))
    }
}

#[cfg(feature = "test")]
fn entries_by_search_text<'a>(text: &'a str) -> impl Fn(&Entry) -> bool + 'a {
    let words = to_words(text);
    move |entry| {
        ((!text.is_empty()
            && words.iter().any(|word| {
                entry.title.to_lowercase().contains(word)
                    || entry.description.to_lowercase().contains(word)
                    || entry.tags.iter().any(|tag| tag == word)
            }))
            || text.is_empty())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn to_words(txt: &str) -> Vec<String> {
        txt.to_lowercase()
            .split(',')
            .map(|x| x.to_string())
            .collect()
    }

    #[test]
    fn is_in_bounding_box() {
        let bb = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, -10.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );
        let e = Entry::build()
            .title("foo")
            .description("bar")
            .lat(5.0)
            .lng(5.0)
            .finish();
        assert_eq!(e.in_bbox(&bb), true);
        let e = Entry::build()
            .title("foo")
            .description("bar")
            .lat(10.1)
            .lng(10.0)
            .finish();
        assert_eq!(e.in_bbox(&bb), false);
    }

    #[test]
    fn filter_by_bounding_box() {
        let bb = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, -10.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );
        let entries = vec![
            Entry::build().lat(5.0).lng(5.0).finish(),
            Entry::build().lat(-5.0).lng(5.0).finish(),
            Entry::build().lat(10.0).lng(10.1).finish(),
        ];
        assert_eq!(
            entries
                .iter()
                .filter(|&x| x.in_bbox(&bb))
                .collect::<Vec<&Entry>>()
                .len(),
            2
        );
    }

    #[test]
    fn filter_by_category() {
        let entries = vec![
            Entry::build().categories(vec!["a"]).finish(),
            Entry::build().categories(vec!["c"]).finish(),
            Entry::build().categories(vec!["b", "a"]).finish(),
        ];
        let ab = vec!["a".into(), "b".into()];
        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(entries_by_category_ids(&ab))
            .collect();
        assert_eq!(x.len(), 2);
        let b = vec!["b".into()];
        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(entries_by_category_ids(&b))
            .collect();
        assert_eq!(x.len(), 1);
        let c = vec!["c".into()];
        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(entries_by_category_ids(&c))
            .collect();
        assert_eq!(x.len(), 1);
    }

    #[test]
    fn filter_by_tags_or_text() {
        let entries = vec![
            Entry::build()
                .id("a")
                .title("aaa")
                .description("bli bla blubb")
                .finish(),
            Entry::build()
                .id("b")
                .title("bbb")
                .description("blabla")
                .tags(vec!["tag1"])
                .finish(),
            Entry::build()
                .id("c")
                .title("ccc")
                .description("bli")
                .tags(vec!["tag2"])
                .finish(),
            Entry::build()
                .id("d")
                .title("ddd")
                .description("blibla")
                .tags(vec!["tag1", "tag2"])
                .finish(),
            Entry::build()
                .id("e")
                .title("eee")
                .description("blubb")
                .description("tag1")
                .finish(),
        ];
        let entries_without_tags = vec![
            Entry::build().id("a").title("a").finish(),
            Entry::build()
                .id("b")
                .title("b")
                .description("blabla")
                .finish(),
            Entry::build().id("c").finish(),
            Entry::build().id("d").finish(),
            Entry::build().id("e").description("tag1").finish(),
        ];
        let tags1 = vec!["tag1".into()];
        let tags2 = vec!["tag1".into(), "tag2".into()];
        let tags3 = vec!["tag2".into()];
        let no_tags = vec![];
        let aaa = "aaa";
        let blabla = "blabla";
        let other = "other";
        let tag1 = "tag1";
        let no_string = "";

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&no_string, &no_tags))
            .collect();
        assert_eq!(x.len(), 5);

        let x: Vec<_> = entries_without_tags
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&other, &tags1))
            .collect();
        assert_eq!(x.len(), 0);

        let x: Vec<_> = entries_without_tags
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&other, &tags2))
            .collect();
        assert_eq!(x.len(), 0);

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&other, &tags2))
            .collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&other, &tags3))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "c");
        assert_eq!(x[1].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&no_string, &tags1))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&aaa, &no_tags))
            .collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, "a");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&aaa, &tags2))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "a");
        assert_eq!(x[1].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&blabla, &tags3))
            .collect();
        assert_eq!(x.len(), 3);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "c");
        assert_eq!(x[2].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&tag1, &no_tags))
            .collect();
        assert_eq!(x.len(), 3);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "d");
        assert_eq!(x[2].id, "e");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&other, &tags1))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "d");
    }

    #[test]
    fn extend_max_bbox() {
        let bbox = MapBbox::new(
            MapPoint::from_lat_lng_deg(-89.99, -179.97),
            MapPoint::from_lat_lng_deg(89.99, 179.97),
        );
        let ext_bbox = extend_bbox(&bbox);
        assert!(ext_bbox.is_valid());
        assert_eq!(ext_bbox.south_west().lat(), LatCoord::min());
        assert_eq!(ext_bbox.north_east().lat(), LatCoord::max());
        assert_eq!(ext_bbox.south_west().lng(), LngCoord::min());
        assert_eq!(ext_bbox.north_east().lng(), LngCoord::max());
    }
}
