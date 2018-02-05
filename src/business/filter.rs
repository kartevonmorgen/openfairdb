use entities::*;
use business::geo::is_in_bbox;

pub trait InBBox {
    fn in_bbox(&self, bb: &Bbox) -> bool;
}

impl InBBox for Entry {
    fn in_bbox(&self, bb: &Bbox) -> bool {
        is_in_bbox(&self.lat, &self.lng, bb)
    }
}

pub fn entries_by_category_ids<'a>(ids: &'a [String]) -> Box<Fn(&Entry) -> bool + 'a> {
    Box::new(move |e| ids.iter().any(|c| e.categories.iter().any(|x| x == c)))
}

pub fn entries_by_tags_or_search_text<'a>(
    text: &'a str,
    tags: &'a [String],
) -> Box<Fn(&Entry) -> bool + 'a> {
    let words = to_words(text);

    if !tags.is_empty() {
        Box::new(move |entry| {
            tags.iter()
                .map(|t| t.to_lowercase())
                .all(|tag| entry.tags.iter().any(|t| *t == tag))
                || ((!text.is_empty() && words.iter().any(|word| {
                    entry.title.to_lowercase().contains(word)
                        || entry.description.to_lowercase().contains(word)
                })) || (text.is_empty() && tags[0] == ""))
        })
    } else {
        Box::new(move |entry| {
            ((!text.is_empty() && words.iter().any(|word| {
                entry.title.to_lowercase().contains(word)
                    || entry.description.to_lowercase().contains(word)
            })) || text.is_empty())
        })
    }
}

fn to_words(txt: &str) -> Vec<String> {
    txt.to_lowercase()
        .split(',')
        .map(|x| x.to_string())
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use business::builder::*;

    #[test]
    fn is_in_bounding_box() {
        let bb = Bbox {
            south_west: Coordinate {
                lat: -10.0,
                lng: -10.0,
            },
            north_east: Coordinate {
                lat: 10.0,
                lng: 10.0,
            },
        };
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
        let bb = Bbox {
            south_west: Coordinate {
                lat: -10.0,
                lng: -10.0,
            },
            north_east: Coordinate {
                lat: 10.0,
                lng: 10.0,
            },
        };
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
            .filter(&*entries_by_category_ids(&ab))
            .collect();
        assert_eq!(x.len(), 2);
        let b = vec!["b".into()];
        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_category_ids(&b))
            .collect();
        assert_eq!(x.len(), 1);
        let c = vec!["c".into()];
        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_category_ids(&c))
            .collect();
        assert_eq!(x.len(), 1);
    }

    #[test]
    fn filter_by_tags_or_text() {
        let entries = vec![
            Entry::build().id("a").title("solawi").finish(),
            Entry::build()
                .id("b")
                .title("blabla")
                .description("bli-blubb")
                .tags(vec!["tag1"])
                .finish(),
            Entry::build().id("c").tags(vec!["tag2"]).finish(),
            Entry::build().id("d").tags(vec!["tag1", "tag2"]).finish(),
            Entry::build().id("e").description("tag1").finish(),
        ];
        let entries_without_tags = vec![
            Entry::build().id("a").title("solawi").finish(),
            Entry::build()
                .id("b")
                .title("blabla")
                .description("bli-blubb")
                .finish(),
            Entry::build().id("c").finish(),
            Entry::build().id("d").finish(),
            Entry::build().id("e").description("tag1").finish(),
        ];
        let tags1 = vec!["tag1".into()];
        let tags2 = vec!["tag1".into(), "tag2".into()];
        let tags3 = vec!["tag2".into()];
        let no_tags = vec![];
        let solawi = "solawi";
        let bliblubb = "bli-blubb";
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
            .filter(&*entries_by_tags_or_search_text(&other, &tags1))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "d");

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
            .filter(&*entries_by_tags_or_search_text(&solawi, &no_tags))
            .collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, "a");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&solawi, &tags2))
            .collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "a");
        assert_eq!(x[1].id, "d");

        let x: Vec<_> = entries
            .iter()
            .cloned()
            .filter(&*entries_by_tags_or_search_text(&bliblubb, &tags3))
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
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, "e");
    }
}
