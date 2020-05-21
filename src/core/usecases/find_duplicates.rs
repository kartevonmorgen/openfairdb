use crate::core::{prelude::*, usecases};
use std::{cmp::min, collections::HashSet};

#[derive(Debug, PartialEq, Serialize)]
pub enum DuplicateType {
    SimilarChars,
    SimilarWords,
}

// Return vector of places like: (entry1ID, entry2ID, reason)
// where entry1 and entry2 are similar places.
pub fn find_duplicates(
    places: &[(Place, ReviewStatus)],
    possible_duplicate_places: &[(Place, ReviewStatus)],
) -> Vec<(Id, Id, DuplicateType)> {
    let mut duplicates = Vec::new();
    for (p1, _) in &places[..] {
        for (p2, _) in &possible_duplicate_places[..] {
            if p1.id >= p2.id {
                continue;
            }
            if let Some(t) = is_duplicate(p1, p2) {
                duplicates.push((p1.id.clone(), p2.id.clone(), t));
            }
        }
    }
    duplicates
}

// Return vector of places like (entryID, reason)
// where the new and yet unregistered place are similar places.
pub fn find_duplicate_of_unregistered_place(
    unregistered_place: &usecases::NewPlace,
    possible_duplicate_places: &[(Place, ReviewStatus)],
) -> Vec<(Id, DuplicateType)> {
    possible_duplicate_places
        .iter()
        .filter_map(|(p, _)| {
            is_duplicate_unregistered_place(&unregistered_place, &p).map(|t| (p.id.clone(), t))
        })
        .collect()
}

const DUPLICATES_MAX_DISTANCE: Distance = Distance::from_meters(100.0);

// returns a DuplicateType if the two places have a similar title and location, otherweise returns None.
fn is_duplicate(e1: &Place, e2: &Place) -> Option<DuplicateType> {
    if is_similar_title(&e1.title, &e2.title, 0.3, 0)
        && is_in_close_proximity_pos(&e1.location.pos, &e2.location.pos, DUPLICATES_MAX_DISTANCE)
    {
        Some(DuplicateType::SimilarChars)
    } else if is_similar_title(&e1.title, &e2.title, 0.0, 2)
        && is_in_close_proximity_pos(&e1.location.pos, &e2.location.pos, DUPLICATES_MAX_DISTANCE)
    {
        Some(DuplicateType::SimilarWords)
    } else {
        None
    }
}

//returns a DuplicateType if the two places have a similar title and location, otherwise returns None.
fn is_duplicate_unregistered_place(
    unregistered_place: &usecases::NewPlace,
    p: &Place,
) -> Option<DuplicateType> {
    if let Some(new_pos) =
        MapPoint::try_from_lat_lng_deg(unregistered_place.lat, unregistered_place.lng)
    {
        if is_similar_title(&unregistered_place.title, &p.title, 0.3, 0)
            && is_in_close_proximity_pos(&new_pos, &p.location.pos, DUPLICATES_MAX_DISTANCE)
        {
            Some(DuplicateType::SimilarChars)
        } else if is_similar_title(&unregistered_place.title, &p.title, 0.0, 2)
            && is_in_close_proximity_pos(&new_pos, &p.location.pos, DUPLICATES_MAX_DISTANCE)
        {
            Some(DuplicateType::SimilarWords)
        } else {
            None
        }
    } else {
        None
    }
}

fn is_in_close_proximity_pos(p1: &MapPoint, p2: &MapPoint, max_dist: Distance) -> bool {
    if let Some(dist) = MapPoint::distance(*p1, *p2) {
        return dist <= max_dist;
    }
    false
}

fn is_similar_title(
    t1: &str,
    t2: &str,
    max_percent_different: f32,
    max_words_different: u32,
) -> bool {
    let max_dist = ((min(t1.len(), t2.len()) as f32 * max_percent_different) + 1.0) as usize; // +1 is to get the ceil

    levenshtein_distance_small(&t1, &t2, max_dist)
        || words_equal_except_k_words(&t1, &t2, max_words_different)
}

// returns true if all but k words are equal in str1 and str2
// (and one of them has more than one word)
// (words in str1 and str2 are treated as sets, order & multiplicity of words doesn't matter)
fn words_equal_except_k_words(str1: &str, str2: &str, k: u32) -> bool {
    let len1 = str1.split_whitespace().count();
    let len2 = str2.split_whitespace().count();

    if (len1 == 1) & (len2 == 1) {
        return false;
    }

    let (s1, s2) = if len1 <= len2 {
        (str1, str2)
    } else {
        (str2, str1)
    };

    let words = s1.split_whitespace();

    let mut diff = 0;
    let mut set = HashSet::new();

    for w in words {
        set.insert(w);
    }

    for w in s2.split(' ') {
        if !set.contains(w) {
            diff += 1;
        }
    }
    diff <= k
}

// Levenshtein Distance more realistically captures typos (all of the following
// operations are counted as distance 1: add one character in between, delete
// one character, change one character)
// but it proved to be way too slow to be run on the whole dataset
fn levenshtein_distance_small(s: &str, t: &str, max_dist: usize) -> bool {
    levenshtein_distance(s, t) <= max_dist
}

// Algorithm from
// https://en.wikipedia.org/wiki/Levenshtein_distance#Computing_Levenshtein_distance
fn levenshtein_distance(s: &str, t: &str) -> usize {
    let max_s: usize = s.len() + 1;
    let max_t: usize = t.len() + 1;

    // for all i and j, d[i,j] will hold the Levenshtein distance between
    // the first i characters of s and the first j characters of t
    // comment that d has (m+1)*(n+1) values
    let mut d: Vec<Vec<usize>> = vec![];
    for _ in 0..max_s {
        d.push(vec![0; max_t]);
    }

    // source (s) prefixes can be transformed into empty string by
    // dropping all characters
    for (i, item) in d.iter_mut().enumerate().take(max_s).skip(1) {
        item[0] = i;
    }

    // target (t) prefixes can be reached from empty source prefix
    // by inserting every character
    for j in 1..max_t {
        d[0][j] = j;
    }

    for j in 1..max_t {
        for i in 1..max_s {
            let substitution_cost = if s.chars().nth(i) == t.chars().nth(j) {
                0
            } else {
                1
            };
            d[i][j] = min3(
                d[i - 1][j] + 1,                     // deletion
                d[i][j - 1] + 1,                     // insertion
                d[i - 1][j - 1] + substitution_cost, // substitution
            )
        }
    }

    d[max_s - 1][max_t - 1]
}

fn min3(s: usize, t: usize, u: usize) -> usize {
    if s <= t {
        min(s, u)
    } else {
        min(t, u)
    }
}

#[cfg(test)]
#[allow(clippy::unreadable_literal, clippy::excessive_precision)]
mod tests {
    use super::*;

    fn new_place(title: String, description: String, pos: MapPoint) -> Place {
        Place::build()
            .id(&title)
            .title(&title)
            .description(&description)
            .pos(pos)
            .finish()
    }

    #[test]
    fn test_in_close_proximity_pos() {
        let pos1 = MapPoint::from_lat_lng_deg(48.23153745093964, 8.003816366195679);
        let pos2 = MapPoint::from_lat_lng_deg(48.23167056421013, 8.003558874130248);

        assert!(is_in_close_proximity_pos(
            &pos1,
            &pos2,
            Distance::from_meters(30.0)
        ));
        assert!(!is_in_close_proximity_pos(
            &pos1,
            &pos2,
            Distance::from_meters(10.0)
        ));
    }

    #[test]
    fn test_similar_title() {
        let e1 = new_place(
            "0123456789".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(48.23153745093964, 6.003816366195679),
        );
        let e2 = new_place(
            "01234567".to_string(),
            "allo! Ein Eintra".to_string(),
            MapPoint::from_lat_lng_deg(48.23153745093964, 6.003816366195679),
        );
        let e3 = new_place(
            "eins zwei drei".to_string(),
            "allo! Ein Eintra".to_string(),
            MapPoint::from_lat_lng_deg(48.23153745093964, 6.003816366195679),
        );
        let e4 = new_place(
            "eins zwei fünf sechs".to_string(),
            "allo! Ein Eintra".to_string(),
            MapPoint::from_lat_lng_deg(48.23153745093964, 6.003816366195679),
        );

        assert_eq!(true, is_similar_title(&e1.title, &e2.title, 0.2, 0)); // only 2 characters changed
        assert_eq!(false, is_similar_title(&e1.title, &e2.title, 0.1, 0)); // more than one character changed
        assert_eq!(true, is_similar_title(&e3.title, &e4.title, 0.0, 2)); // only 2 words changed
        assert_eq!(false, is_similar_title(&e3.title, &e4.title, 0.0, 1)); // more than 1 word changed
    }
    #[test]
    fn test_is_duplicate_unregistered_place() {
        let x = &usecases::NewPlace {
            title: "Ein Eintrag Blablabla".into(),
            description: "Hallo! Ein Eintrag".into(),
            lat: 47.23153745093964,
            lng: 5.003816366195679,
            street: None,
            zip: None,
            city: None,
            country: None,
            state: None,
            email: None,
            telephone: None,
            homepage: None,
            opening_hours: None,
            categories: vec![],
            tags: vec![],
            license: "CC0-1.0".into(),
            image_url: None,
            image_link_url: None,
        };

        let e2 = new_place(
            "Eintrag".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093970, 5.003816366195679),
        );
        let e3 = new_place(
            "Enn Eintrxg Blablalx".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let e4 = new_place(
            "En Eintrg Blablala".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let e5 = new_place(
            "Ein Eintrag Blabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(40.23153745093960, 5.003816366195670),
        );
        let e6 = new_place(
            "Ein Eintrag Blablabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093964, 5.003816366195679),
        );

        // titles have a word that is equal
        assert_eq!(
            Some(DuplicateType::SimilarWords),
            is_duplicate_unregistered_place(&x, &e2)
        );
        // titles similar: small hamming distance
        assert_eq!(
            Some(DuplicateType::SimilarChars),
            is_duplicate_unregistered_place(&x, &e3)
        );
        // titles similar: small levenshtein distance
        assert_eq!(
            Some(DuplicateType::SimilarChars),
            is_duplicate_unregistered_place(&x, &e4)
        );
        // exact_same
        assert_eq!(
            Some(DuplicateType::SimilarChars),
            is_duplicate_unregistered_place(&x, &e6)
        );
        // too far away
        assert_eq!(None, is_duplicate_unregistered_place(&x, &e5));
    }

    #[test]
    fn test_is_duplicate() {
        let e1 = new_place(
            "Ein Eintrag Blablabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093964, 5.003816366195679),
        );
        let e2 = new_place(
            "Eintrag".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093970, 5.003816366195679),
        );
        let e3 = new_place(
            "Enn Eintrxg Blablalx".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let e4 = new_place(
            "En Eintrg Blablala".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let e5 = new_place(
            "Ein Eintrag Blabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(40.23153745093960, 5.003816366195670),
        );

        // titles have a word that is equal
        assert_eq!(Some(DuplicateType::SimilarWords), is_duplicate(&e1, &e2));
        // titles similar: small levenshtein distance
        assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&e1, &e4));
        // titles similar: small hamming distance
        assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&e1, &e3));
        // titles not similar
        assert_eq!(None, is_duplicate(&e2, &e4));
        // places not located close together
        assert_eq!(None, is_duplicate(&e4, &e5));
    }

    #[test]
    fn test_min() {
        assert_eq!(1, min3(1, 2, 3));
        assert_eq!(2, min3(3, 2, 3));
        assert_eq!(2, min3(3, 3, 2));
        assert_eq!(1, min3(1, 1, 1));
    }

    #[test]
    fn test_words_equal() {
        assert_eq!(true, words_equal_except_k_words("ab abc a", "ab abc b", 1));
        assert_eq!(true, words_equal_except_k_words("ab abc a", "abc ab", 1));
        assert_eq!(true, words_equal_except_k_words("ab ac a", "abc ab ab", 2));
        assert_eq!(false, words_equal_except_k_words("a a a", "ab abc", 2));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(3, levenshtein_distance("012a34c", "0a3c")); // delete 1,2 and 4
        assert_eq!(1, levenshtein_distance("12345", "a12345")); // insert a
        assert_eq!(1, levenshtein_distance("aabaa", "aacaa")); // replace b by c
    }
}
