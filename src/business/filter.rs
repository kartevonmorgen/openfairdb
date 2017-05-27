use entities::*;
use super::geo::Coordinate;

pub trait InBBox {
    fn in_bbox(&self, bb: &[Coordinate]) -> bool;
}

pub enum Combination {
    And,
    Or
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

pub fn entries_by_category_ids<'a>(ids: &'a [String]) -> Box<Fn(&&Entry) -> bool + 'a> {
    Box::new(move |e| ids.iter().any(|c| e.categories.iter().any(|x| x == c)))
}

pub fn triple_by_subject<'a>(o_id : ObjectId) -> Box<Fn(&&Triple) -> bool + 'a> {
    Box::new(move |triple| o_id == triple.subject)
}

pub fn entries_by_tags<'a>(tags: &'a [String], triples: &'a [Triple], combination: Combination) -> Box<Fn(&&Entry) -> bool + 'a> {


    let triples : Vec<(&String, &String)> = triples
        .into_iter()
        .filter_map(|x| match *x {
            Triple {
                subject   : ObjectId::Entry(ref e_id),
                predicate : Relation::IsTaggedWith,
                object    : ObjectId::Tag(ref t_id)
            } => Some((e_id,t_id)),
            _ => None
        })
        .collect();

    match combination {
        Combination::Or => {
            Box::new(move |entry|
                tags.iter()
                    .any(|tag| triples.iter().any(|t| *t.0 == entry.id && t.1 == tag))
            )
        },
        Combination::And => {
            Box::new(move |entry| {
                let e_tags : Vec<&String> = triples
                    .iter()
                    .filter(|t| *t.0 == entry.id)
                    .map(|t|t.1)
                    .collect();
                    tags.iter().all(|tag|e_tags.iter().any(|t| *t == tag))
                }
            )
        }
    }
}

pub fn entries_by_search_text<'a>(text: &'a str) -> Box<Fn(&&Entry) -> bool + 'a> {
    let words = to_words(text);
    Box::new(move |e| {
        words.iter().any(|word| {
            e.title.to_lowercase().contains(word) || e.description.to_lowercase().contains(word)
        })
    })
}

pub fn entries_by_tags_or_search_text<'a>(text: &'a str, tags: &'a [String], triples: &'a [Triple]) -> Box<Fn(&&Entry) -> bool + 'a> {
    let tag_triples : Vec<(&String, &String)> = triples
        .into_iter()
        .filter_map(|x| match *x {
            Triple {
                subject   : ObjectId::Entry(ref e_id),
                predicate : Relation::IsTaggedWith,
                object    : ObjectId::Tag(ref t_id)
            } => Some((e_id,t_id)),
            _ => None
        })
        .collect();

    let words = to_words(text);

    Box::new(move |entry|
        tags.iter()
            .any(|tag| tag_triples.iter().any(|t| *t.0 == entry.id && t.1 == tag))
        || words.iter().any(|word| {
            entry.title.to_lowercase().contains(word) || entry.description.to_lowercase().contains(word)
        })
    )
}

fn to_words(txt: &str) -> Vec<String> {
    txt.to_lowercase().split(',').map(|x| x.to_string()).collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use business::builder::*;

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
    fn is_in_invalid_bounding_box() {
        let bb = vec![Coordinate {
                          lat: 10.0,
                          lng: 10.0,
                      }];
        let e = Entry::build()
            .lat(5.0)
            .lng(5.0)
            .finish();
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
        let entries = vec![
            Entry::build().lat(5.0).lng(5.0).finish(),
            Entry::build().lat(-5.0).lng(5.0).finish(),
            Entry::build().lat(10.0).lng(10.1).finish(),
        ];
        assert_eq!(entries.iter().filter(|&x| x.in_bbox(&bb)).collect::<Vec<&Entry>>().len(),
                   2);
    }

    #[test]
    fn filter_by_category() {
        let entries = vec![
            Entry::build().categories(vec!["a"]).finish(),
            Entry::build().categories(vec!["c"]).finish(),
            Entry::build().categories(vec!["b","a"]).finish(),
        ];
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
        let entries = vec![
            Entry::build().title("a title").description("x").finish(),
            Entry::build().title("not so interesting").description("y").finish(),
        ];
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
        let entries = vec![
            Entry::build().title("a").description("x").finish(),
            Entry::build().title("b").description("y").finish(),
            Entry::build().title("c").description("x").finish(),
        ];
        let filter = entries_by_search_text("x");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
    }

    #[test]
    fn search_with_multiple_words() {
        let entries = vec![
            Entry::build().title("SoLaWi").description("mit gemüse").finish(),
            Entry::build().title("csa").description("This is a great csa").finish(),
            Entry::build().title("solawi").description("Das ist eine tolle solawi").finish(),
        ];
        let filter = entries_by_search_text("csa,toll");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 2);
        let filter = entries_by_search_text("great,to,gemü");
        let x: Vec<&Entry> = entries.iter().filter(&*filter).collect();
        assert_eq!(x.len(), 3);
    }

    #[test]
    fn search_and_ignore_capitalisation() {
        let e0 = Entry::build().title("Eintrag").description("Hallo! Ein EinTrag").finish();
        let e1 = Entry::build().title("Ein trag").description("foo").finish();
        let e2 = Entry::build().title("CSA").description("cool vegetables").finish();
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
    fn filter_by_tags() {
        let entries = vec![
            Entry::build().id("a").finish(),
            Entry::build().id("b").finish(),
            Entry::build().id("c").finish(),
        ];
        let tags = vec!["csa".into()];
        let no_triples = vec![];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&no_triples, Combination::Or)).collect();
        assert_eq!(x.len(), 0);
        let triples = vec![
            Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
            Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo".into())}
        ];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, Combination::Or)).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id,"b");
        let tags = vec!["csa".into(),"foo".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, Combination::Or)).collect();
        assert_eq!(x.len(), 2);
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, Combination::And)).collect();
        assert_eq!(x.len(), 0);
    }

     #[test]
    fn filter_by_tags_or_text() {
        let entries = vec![
            Entry::build().id("a").title("solawi").finish(),
            Entry::build().id("b").title("blabla").description("blubb").finish(),
            Entry::build().id("c").finish(),
        ];
        let tags = vec!["csa".into()];
        let solawi = "solawi";
        let blubb = "blubb";
        let slowtec = "slowtec";
        
        let no_triples = vec![];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&slowtec, &tags, &no_triples)).collect();
        assert_eq!(x.len(), 0);

        let triples = vec![
            Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
            Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo".into())}
        ];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&slowtec, &tags, &triples)).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id,"b");
        let tags = vec!["csa".into(),"foo".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&slowtec, &tags, &triples)).collect();
        assert_eq!(x.len(), 2);
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&slowtec, &tags, &triples)).collect();
        assert_eq!(x.len(), 0);

        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&slowtec, &tags, &no_triples)).collect();
        assert_eq!(x.len(), 0);

        let triples = vec![
            Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
            Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo".into())}
        ];

        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&solawi, &tags, &triples)).collect();
        assert_eq!(x[0].id,"a");
        assert_eq!(x[1].id,"b");
        assert_eq!(x.len(), 2);

        let tags = vec!["foo".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags_or_search_text(&blubb, &tags, &triples)).collect();
        assert_eq!(x.len(), 2);
        assert_eq!(x[0].id, "b");
        assert_eq!(x[1].id, "c");
    }
}
