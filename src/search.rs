// Copyright (c) 2015 Markus Kohlhase <mail@markus-kohlhase.de>

use json::{Entry, Category};

pub trait Search {
  fn filter_by_search_text(&self, text:&String) -> Self;
  fn map_to_ids(&self) -> Vec<String>;
}

impl Search for Vec<Entry> {
  fn filter_by_search_text(&self, text:&String) -> Vec<Entry> {
    by_text(self,text,entry_filter_factory)
  }
  fn map_to_ids(&self) -> Vec<String>{
    self.iter().filter_map(|e|e.clone().id).map(|x|x.clone()).collect()
  }
}

impl Search for Vec<Category> {
  fn filter_by_search_text(&self, text:&String) -> Vec<Category> {
    by_text(self,text,category_filter_factory)
  }
  fn map_to_ids(&self) -> Vec<String>{
    self.iter().filter_map(|e|e.clone().id).map(|x|x.clone()).collect()
  }

}

fn by_text<'a,T:Clone,F>(collection: &'a Vec<T>, text:&String, filter:F) -> Vec<T>
where F:Fn(&'a T) -> Box<Fn(&str) -> bool+'a>
{
  let txt_cpy = text.clone().to_lowercase();
  let words = txt_cpy.split(',');
  collection.iter()
    .filter(|&e|{
      let f = filter(e);
      words.clone().any(|word|f(&word))
    })
    .map(|x|x.clone())
    .collect()
}

fn entry_filter_factory<'a>(e:&'a Entry) -> Box<Fn(&str) -> bool + 'a> {
  Box::new(move |word|
    e.title      .to_lowercase().contains(word) ||
    e.description.to_lowercase().contains(word)
  )
}

fn category_filter_factory<'a>(e:&'a Category) -> Box<Fn(&str) -> bool + 'a> {
  Box::new(move |word| match e.name {
    Some(ref n) => n.to_lowercase().contains(word),
    None => false
  })
}
