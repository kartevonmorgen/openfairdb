// Copyright (c) 2015 Markus Kohlhase <mail@markus-kohlhase.de>

use geo;
use json::{Entry, Category};
use geo::Coordinate;
use std::cmp::min;
use std::collections::HashSet;


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

// return vector of entries like: (entry1ID, entry2ID, reason) where entry1 and entry2 are similar entries
pub fn find_duplicates(entries: &Vec<Entry>) -> Vec<(&String, &String, DuplicateType)> {
  let mut duplicates = Vec::new();
  for i in 0..entries.len() {
    for j in (i+1)..entries.len() {
      if let Some(t) = is_duplicate(&entries[i], &entries[j]) {
        if let Some(ref id1) = entries[i].id {
          if let Some(ref id2) = entries[j].id {
            duplicates.push((id1, id2, t));
          }
        }
      }
    }
  }
  duplicates
}

#[derive(Debug, PartialEq, RustcEncodable)]
pub enum DuplicateType {
    SimilarChars,
    SimilarWords
}

// returns a DuplicateType if the two entries have a similar title, returns None otherwise
fn is_duplicate(e1 : &Entry, e2: &Entry) -> Option<DuplicateType>{
  if similar_title(&e1, &e2, 0.3, 0) && in_close_proximity(&e1, &e2, 100.0) {
    Some(DuplicateType::SimilarChars)
  } else if similar_title(&e1, &e2, 0.0, 2) && in_close_proximity(&e1, &e2, 100.0) {
    Some(DuplicateType::SimilarWords)
  } else {
    None
  }
}

fn in_close_proximity(e1: &Entry, e2: &Entry, max_dist_meters:f64) -> bool {
    entry_distance_in_meters(&e1, &e2) <= max_dist_meters
}

fn entry_distance_in_meters(e1: &Entry, e2: &Entry) -> f64 {
  let coord1 = Coordinate{lat: e1.lat, lng:e1.lng};
  let coord2 = Coordinate{lat: e2.lat, lng:e2.lng};
  geo::distance(&coord1, &coord2) * 1000.0
}

fn similar_title(e1: &Entry, e2: &Entry, max_percent_different: f32, max_words_different: u32) -> bool {
  let max_dist : usize = ((min(e1.title.len(),e2.title.len()) as f32 * max_percent_different) + 1.0) as usize;  // +1 is to get the ceil

  levenshtein_distance_small(&e1.title, &e2.title, max_dist)
  || words_equal_except_k_words(&e1.title, &e2.title, max_words_different)
}

// returns true if all but k words are equal in str1 and str2 (and one of them has more than one word)
// (words in str1 and str2 are treated as sets, order & multiplicity of words doesn't matter)
fn words_equal_except_k_words(str1: &str, str2:&str, k:u32) -> bool {

  let len1 =  str1.split_whitespace().count();
  let len2 =  str2.split_whitespace().count();

  if (len1 == 1) & (len2 == 1) {
    return false;
  }

  let s1 : &str;
  let s2 : &str;

  if len1 <= len2 {
    s1 = str1;
    s2 = str2;
  } else{
    s1 = str2;
    s2 = str1;
  }

  let words = s1.split_whitespace();

  let mut diff = 0;
  let mut set = HashSet::new();

  for w in words {
    set.insert(w);
  }

  for w in s2.split(" ") {
    if !set.contains(w) {
      diff = diff + 1;
    }
  }
  diff <= k
}

// returns true if the hamming distance between str1 and str2 is smaller or
// equal as maxDist (doesn't need to calculate the full hamming distance because
// it aborts as soon as maxDist is reached)
fn hamming_distance_small(str1: &str, str2:&str, max_dist:usize) -> bool{
  let mut dist = 0;
  for i in 0..str1.chars().count() {
    if str1.chars().nth(i) != str2.chars().nth(i) {
      dist = dist + 1;
      if dist > max_dist {
        break;
      }
    }
  }
  dist <= max_dist
}

// Levenshtein Distance more realistically captures typos (all of the following
// operations are counted as distance 1: add one character in between, delete
// one character, change one character)
// but it proved to be way too slow to be run on the whole dataset
fn levenshtein_distance_small(s: &str, t:&str, max_dist: usize) -> bool{
  levenshtein_distance(s, t) <= max_dist
}

// Algorithm from https://en.wikipedia.org/wiki/Levenshtein_distance#Computing_Levenshtein_distance
pub fn levenshtein_distance(s: &str, t: &str) -> usize {
  let max_s : usize = s.len() + 1;
  let max_t : usize = t.len() + 1;

  // for all i and j, d[i,j] will hold the Levenshtein distance between
  // the first i characters of s and the first j characters of t
  // note that d has (m+1)*(n+1) values
  let mut d: Vec<Vec<usize>> = vec![];
  for i in 0..max_s{
    d.push(vec![0;max_t]);
  }

  // source (s) prefixes can be transformed into empty string by
  // dropping all characters
  for i  in 1..max_s {
    d[i][0] = i;
  }

  // target (t) prefixes can be reached from empty source prefix
  // by inserting every character
  for j in 1..max_t {
      d[0][j] = j;
  }

  for j in 1..max_t {
      for i in 1..max_s{
        let substitutionCost = if s.chars().nth(i) == t.chars().nth(j) { 
          0 
        } else { 
          1 
        };
        d[i][j] = min3(d[i-1][j] + 1,                   // deletion
                       d[i][j-1] + 1,                   // insertion
                       d[i-1][j-1] + substitutionCost)  // substitution
      }
  }

  d[max_s-1][max_t-1]
}

fn min3(s:usize, t:usize, u:usize) -> usize {
  if s <= t {
    min(s,u)
  } else{
    min(t,u)
  }
}

// TESTS:
impl Entry {
  fn new(title: String, description: String, lat: f64, lng:f64) -> Entry{
    Entry {
      id          : None,
      created     : None,
      version     : None,
      title       : title,
      description : description,
      lat         : lat,
      lng         : lng,
      street      : None,
      zip         : None,
      city        : None,
      country     : None,
      email       : None,
      telephone   : None,
      homepage    : None,
      categories  : None,
      license     : None
    }
  }
}

#[test]
fn test_hamming_distance_small(){
  assert_eq!(true, hamming_distance_small("aaaaa", "aabab", 2));
  assert_eq!(false, hamming_distance_small("aaaaa", "abbba", 2));
  assert_eq!(true, hamming_distance_small("aaaaaa", "abaaa", 2));
  assert_eq!(false, hamming_distance_small("aaaaaa", "abaa", 2));
  assert_eq!(false, hamming_distance_small("Hallo! Ein Eintrag", "Hallo! Tschüss", 4));
}

#[test]
fn test_in_close_proximity(){
  let e1 = Entry::new("Entry 1".to_string(), "Punkt1".to_string(), 48.23153745093964, 8.003816366195679);
  let e2 = Entry::new("Entry 2".to_string(), "Punkt2".to_string(), 48.23167056421013, 8.003558874130248);

  assert_eq!(in_close_proximity(&e1, &e2, 30.0), true);
  assert_eq!(in_close_proximity(&e1, &e2, 10.0), false);
}

#[test]
fn test_similar_title(){
  let e1 = Entry::new("0123456789".to_string(), "Hallo! Ein Eintrag".to_string(), 48.23153745093964, 6.003816366195679);
  let e2 = Entry::new("01234567".to_string(), "allo! Ein Eintra".to_string(), 48.23153745093964, 6.003816366195679);
  let e3 = Entry::new("eins zwei drei".to_string(), "allo! Ein Eintra".to_string(), 48.23153745093964, 6.003816366195679);
  let e4 = Entry::new("eins zwei fünf sechs".to_string(), "allo! Ein Eintra".to_string(), 48.23153745093964, 6.003816366195679);

  assert_eq!(true, similar_title(&e1, &e2, 0.2, 0));  // only 2 characters changed
  assert_eq!(false, similar_title(&e1, &e2, 0.1, 0)); // more than one character changed
  assert_eq!(true, similar_title(&e3, &e4, 0.0, 2));  // only 2 words changed
  assert_eq!(false, similar_title(&e3, &e4, 0.0, 1)); // more than 1 word changed
}

#[test]
fn test_is_duplicate(){
  let e1 = Entry::new("Ein Eintrag Blablabla".to_string(), "Hallo! Ein Eintrag".to_string(), 47.23153745093964, 5.003816366195679);
  let e2 = Entry::new("Eintrag".to_string(), "Hallo! Ein Eintrag".to_string(), 47.23153745093970, 5.003816366195679);
  let e3 = Entry::new("Enn Eintrxg Blablalx".to_string(), "Hallo! Ein Eintrag".to_string(), 47.23153745093955, 5.003816366195679);
  let e4 = Entry::new("En Eintrg Blablala".to_string(), "Hallo! Ein Eintrag".to_string(), 47.23153745093955, 5.003816366195679);
  let e5 = Entry::new("Ein Eintrag Blabla".to_string(), "Hallo! Ein Eintrag".to_string(), 40.23153745093960, 5.003816366195670);

  assert_eq!(Some(DuplicateType::SimilarWords), is_duplicate(&e1, &e2));  // titles have a word that is equal
  assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&e1, &e4));  // titles similar: small levenshtein distance
  assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&e1, &e3));  // titles similar: small hamming distance
  assert_eq!(None, is_duplicate(&e2, &e4));     // titles not similar
  assert_eq!(None, is_duplicate(&e4, &e5));     // entries not located close together
}

#[test]
fn test_min(){
  assert_eq!(1, min3(1,2,3));
  assert_eq!(2, min3(3,2,3));
  assert_eq!(2, min3(3,3,2));
  assert_eq!(1, min3(1,1,1));
}

#[test]
fn test_words_equal(){
  assert_eq!(true, words_equal_except_k_words("ab abc a", "ab abc b", 1));
  assert_eq!(true, words_equal_except_k_words("ab abc a", "abc ab", 1));
  assert_eq!(true, words_equal_except_k_words("ab ac a", "abc ab ab", 2));
  assert_eq!(false, words_equal_except_k_words("a a a", "ab abc", 2));
}

#[test]
fn test_levenshtein_distance(){
  assert_eq!(3, levenshtein_distance("012a34c", "0a3c"));   // delete 1,2 and 4
  assert_eq!(1, levenshtein_distance("12345", "a12345")); // insert a
  assert_eq!(1, levenshtein_distance("aabaa", "aacaa"));    // replace b by c
}
