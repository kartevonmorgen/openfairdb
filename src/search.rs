// Copyright (c) 2015 Markus Kohlhase <mail@markus-kohlhase.de>

use json::{Entry, Category};

pub trait Search {
    fn filter_by_search_text(&self, text: &str) -> Self;
    fn map_to_ids(&self) -> Vec<String>;
}

impl Search for Vec<Entry> {
    fn filter_by_search_text(&self, text: &str) -> Vec<Entry> {
        by_text(self, text, entry_filter_factory)
    }
    fn map_to_ids(&self) -> Vec<String> {
        self.iter().cloned().filter_map(|e| e.clone().id).collect()
    }
}

impl Search for Vec<Category> {
    fn filter_by_search_text(&self, text: &str) -> Vec<Category> {
        by_text(self, text, category_filter_factory)
    }
    fn map_to_ids(&self) -> Vec<String> {
        self.iter().cloned().filter_map(|e| e.clone().id).collect()
    }
}

fn by_text<'a, T: Clone, F>(collection: &'a [T], text: &str, filter: F) -> Vec<T>
    where F: Fn(&'a T) -> Box<Fn(&str) -> bool + 'a>
{
    let txt_cpy = text.to_lowercase();
    let words = txt_cpy.split(',');
    collection.iter()
        .filter(|&e| {
            let f = filter(e);
            words.clone().any(|word| f(word))
        })
        .cloned()
        .collect()
}

fn entry_filter_factory<'a>(e: &'a Entry) -> Box<Fn(&str) -> bool + 'a> {
    Box::new(move |word| {
        e.title.to_lowercase().contains(word) || e.description.to_lowercase().contains(word)
    })
}

fn category_filter_factory<'a>(e: &'a Category) -> Box<Fn(&str) -> bool + 'a> {
    Box::new(move |word| match e.name {
        Some(ref n) => n.to_lowercase().contains(word),
        None => false,
    })
}
