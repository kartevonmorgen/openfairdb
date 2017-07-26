
pub enum Combination {
    And,
    Or
}

pub fn entries_by_tags<'a>(tags: &'a [String], triples: &'a [Triple], combination: &Combination) -> Box<Fn(&&Entry) -> bool + 'a> {
    let triples: Vec<(&String, &String)> = triples
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

    match *combination {
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

#[cfg(test)]
mod tests {

    use super::*;
    use business::builder::*;

     #[test]
    fn filter_by_tags() {
        let entries = vec![
            Entry::build().id("a").finish(),
            Entry::build().id("b").finish(),
            Entry::build().id("c").finish(),
        ];
        let tags = vec!["csa".into()];
        let no_triples = vec![];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&no_triples, &Combination::Or)).collect();
        assert_eq!(x.len(), 0);
        let triples = vec![
            Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
            Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo".into())}
        ];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, &Combination::Or)).collect();
        assert_eq!(x.len(), 1);
        assert_eq!(x[0].id,"b");
        let tags = vec!["csa".into(),"foo".into()];
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, &Combination::Or)).collect();
        assert_eq!(x.len(), 2);
        let x: Vec<&Entry> = entries.iter().filter(&*entries_by_tags(&tags,&triples, &Combination::And)).collect();
        assert_eq!(x.len(), 0);
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
}