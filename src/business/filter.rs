use entities::*;
use super::geo::Coordinate;

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

pub fn entries_by_category_ids<'a>(ids: &'a Vec<String>) -> Box<Fn(&&Entry) -> bool + 'a> {
    Box::new(move |e| ids.iter().any(|c| e.categories.iter().any(|x| x == c)))
}

pub fn entries_by_search_text<'a>(text: &'a str) -> Box<Fn(&&Entry) -> bool + 'a> {
    let words = to_words(text);
    Box::new(move |e| {
        words.iter().any(|word| {
            e.title.to_lowercase().contains(word) || e.description.to_lowercase().contains(word)
        })
    })
}

pub fn categories_by_search_text<'a>(text: &'a str) -> Box<Fn(&&Category) -> bool + 'a> {
    let words = to_words(text);
    Box::new(move |c| words.iter().any(|word| c.name.to_lowercase().contains(word)))
}

fn to_words(txt: &str) -> Vec<String> {
    txt.to_lowercase().split(',').map(|x| x.to_string()).collect()
}

#[cfg(test)]
mod tests {

    use super::*;

    fn new_entry(title: &str, description: &str, lat: f64, lng: f64, cats: Option<Vec<String>>) -> Entry {
        Entry{
            id          : title.clone().into(),
            created     : 0,
            version     : 0,
            title       : title.into(),
            description : description.into(),
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

    fn new_category(name: &str) -> Category {
            Category {
                id        : name.clone().into(),
                created   : 0,
                version   : 0,
                name      : name.into()
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
        let e = new_entry("foo", "bar", 5.0, 5.0, None);
        assert_eq!(e.in_bbox(&bb), true);
        let e = new_entry("foo", "bar", 10.1, 10.0, None);
        assert_eq!(e.in_bbox(&bb), false);
    }

    #[test]
    fn is_in_invalid_bounding_box() {
        let bb = vec![Coordinate {
                          lat: 10.0,
                          lng: 10.0,
                      }];
        let e = new_entry("", "", 5.0, 5.0, None);
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
        let entries = vec![new_entry("", "", 5.0, 5.0, None),
                           new_entry("", "", -5.0, 5.0, None),
                           new_entry("", "", 10.0, 10.1, None)];
        assert_eq!(entries.iter().filter(|&x| x.in_bbox(&bb)).collect::<Vec<&Entry>>().len(),
                   2);
    }

    #[test]
    fn filter_by_category() {
        let entries = vec![new_entry("", "", 5.0, 5.0, Some(vec!["a".into()])),
                           new_entry("", "", -5.0, 5.0, Some(vec!["c".into()])),
                           new_entry("", "", 10.0, 10.1, Some(vec!["b".into(), "a".into()]))];
        let ab = vec!["a".into(), "b".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_category_ids(&ab)).collect();
        assert_eq!(x.len(), 2);
        let b = vec!["b".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_category_ids(&b)).collect();
        assert_eq!(x.len(), 1);
        let c = vec!["c".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_category_ids(&c)).collect();
        assert_eq!(x.len(), 1);
    }

    #[test]
    fn search_entries_by_title() {
        let entries = vec![new_entry("a title", "x".into(), 0.0, 0.0, None),
                           new_entry("not so interesting", "y", 0.0, 0.0, None)];
        let filter = entries_by_search_text("a");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].title, "a title");
        let filter = entries_by_search_text("ti");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
    }

    #[test]
    fn search_entries_by_description() {
        let entries = vec![new_entry("a", "x", 0.0, 0.0, None),
                           new_entry("b", "y", 0.0, 0.0, None),
                           new_entry("c", "x", 0.0, 0.0, None)];
        let filter = entries_by_search_text("x");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
    }

    #[test]
    fn search_with_multiple_words() {
        let entries = vec![new_entry("SoLaWi", "mit gemüse", 0.0, 0.0, None),
                           new_entry("csa", "This is a great csa", 0.0, 0.0, None),
                           new_entry("solawi", "Das ist eine tolle solawi", 0.0, 0.0, None)];
        let filter = entries_by_search_text("csa,toll");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
        let filter = entries_by_search_text("great,to,gemü");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 3);
    }

    #[test]
    fn search_and_ignore_capitalisation() {
        let e0 = new_entry("Eintrag", "Hallo! Ein EinTrag", 0.0, 0.0, None);
        let e1 = new_entry("Ein trag", "foo", 0.0, 0.0, None);
        let e2 = new_entry("CSA", "cool vegetables", 0.0, 0.0, None);
        let entries = vec![&e0, &e1, &e2];

        let filter = entries_by_search_text("Foo");
        let x: Vec<&Entry> = entries.iter().cloned().filter(&*filter).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, e1.id);

        let filter = entries_by_search_text("trag");
        let x: Vec<&Entry> = entries.iter().cloned().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, e0.id);
        assert_eq!(x[1].id, e1.id);

        let filter = entries_by_search_text("csa");
        let x: Vec<&Entry> = entries.iter().cloned().filter(&*filter).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id, e2.id);
    }

    #[test]
    fn search_categories() {
        let c0 = new_category("Foo");
        let c1 = new_category("Bar");
        let c2 = new_category("baz");
        let cats = vec![&c0, &c1, &c2];

        let filter = categories_by_search_text("foo");
        let x: Vec<&Category> = cats.iter().cloned().filter(&*filter).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].name, c0.name);

        let filter = categories_by_search_text("ar");
        let x: Vec<&Category> = cats.iter().cloned().filter(&*filter).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].name, "Bar");
        let filter = categories_by_search_text("az");
        let x: Vec<&Category> = cats.iter().cloned().filter(&*filter).collect();
        assert_eq!(x[0].name, "baz");
    }

}
