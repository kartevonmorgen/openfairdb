use std::{cmp::min, collections::HashSet};

use crate::core::{prelude::*, usecases::NewPlace};

#[derive(Debug, PartialEq, Serialize)]
pub enum DuplicateType {
    SimilarChars,
    SimilarWords,
}

// Return vector of places like: (entry1ID, entry2ID, reason)
// where entry1 and entry2 are similar places.
pub fn find_duplicates(
    place_index: &dyn PlaceIndex,
    places: &[(Place, ReviewStatus)],
) -> Result<Vec<(Id, Id, DuplicateType)>> {
    let mut duplicates = Vec::new();
    for (p1, _) in places {
        let nearby_places = search_nearby_places(place_index, p1.location.pos)?;
        for p2 in nearby_places {
            if let Some(t) = is_duplicate(p1, &p2) {
                duplicates.push((p1.id.clone(), Id::from(p2.id), t));
            }
        }
    }
    Ok(duplicates)
}

pub fn retain_duplicates_of(
    nearby_places: Vec<IndexedPlace>,
    new_place: &NewPlace,
) -> Vec<IndexedPlace> {
    nearby_places
        .into_iter()
        .filter(|p| is_duplicate_of(new_place, p))
        .collect()
}

const MAX_NEARBY_RESULTS: usize = 1000;

const MAX_NEARBY_RADIUS: Distance = Distance::from_meters(100.0);

const MAX_NEARBY_DIAMETER: Distance = Distance::from_meters(MAX_NEARBY_RADIUS.to_meters() * 2.0);

const MAX_TEXT_RELATIVE_EDIT_DISTANCE: f64 = 0.3; // max. 30% text difference

const MAX_WORDS_HAMMING_DISTANCE: u32 = 2; // up to 2 words may differ

fn search_nearby_places(
    place_index: &dyn crate::core::db::PlaceIndex,
    center: MapPoint,
) -> Result<Vec<IndexedPlace>> {
    let nearby_bbox = nearby_bbox(center);
    let nearby_query = crate::core::db::IndexQuery {
        include_bbox: Some(nearby_bbox),
        ..Default::default()
    };
    Ok(place_index
        .query_places(&nearby_query, MAX_NEARBY_RESULTS)
        .map_err(RepoError::Other)?)
}

pub fn search_duplicates(
    place_index: &dyn crate::core::db::PlaceIndex,
    new_place: &NewPlace,
) -> Result<Vec<IndexedPlace>> {
    let center = MapPoint::new(
        LatCoord::from_deg(new_place.lat),
        LngCoord::from_deg(new_place.lng),
    );
    let nearby_places = search_nearby_places(place_index, center)?;
    Ok(retain_duplicates_of(nearby_places, new_place))
}

pub fn nearby_bbox(center: MapPoint) -> MapBbox {
    MapBbox::centered_around(center, MAX_NEARBY_DIAMETER, MAX_NEARBY_DIAMETER)
}

// returns a DuplicateType if the two places have a similar title and location,
// otherwise returns None.
fn is_duplicate(e1: &Place, e2: &IndexedPlace) -> Option<DuplicateType> {
    if e1.id.as_str() == e2.id.as_str() {
        // Skip identical places
        return None;
    }
    if !is_in_close_proximity_pos(&e1.location.pos, &e2.pos, MAX_NEARBY_RADIUS) {
        return None;
    }
    if is_similar_text(&e1.title, &e2.title, MAX_TEXT_RELATIVE_EDIT_DISTANCE, 0) {
        return Some(DuplicateType::SimilarChars);
    }
    if is_similar_text(&e1.title, &e2.title, 0.0, MAX_WORDS_HAMMING_DISTANCE) {
        return Some(DuplicateType::SimilarWords);
    }
    None
}

fn is_similar_text(
    text1: &str,
    text2: &str,
    max_text_relative_edit_distance: f64,
    max_words_hamming_distance: u32,
) -> bool {
    let max_levenshtein_dist =
        (min(text1.len(), text2.len()) as f64 * max_text_relative_edit_distance).ceil() as usize;
    levenshtein_distance_small(text1, text2, max_levenshtein_dist)
        || words_equal_except_k_words(text1, text2, max_words_hamming_distance)
}

fn is_duplicate_of(new_place: &NewPlace, indexed_place: &IndexedPlace) -> bool {
    let pos = MapPoint::from_lat_lng_deg(new_place.lat, new_place.lng);
    if !is_in_close_proximity_pos(&pos, &indexed_place.pos, MAX_NEARBY_RADIUS) {
        return false;
    }
    // TODO: Compare more (text) fields than just the title?
    if !is_similar_text(
        &new_place.title,
        &indexed_place.title,
        MAX_TEXT_RELATIVE_EDIT_DISTANCE,
        MAX_WORDS_HAMMING_DISTANCE,
    ) {
        return false;
    }
    true
}

fn is_in_close_proximity_pos(p1: &MapPoint, p2: &MapPoint, max_dist: Distance) -> bool {
    if let Some(dist) = MapPoint::distance(*p1, *p2) {
        return dist <= max_dist;
    }
    false
}

// returns true if all but k words are equal in str1 and str2
// (and one of them has more than one word)
// (words in str1 and str2 are treated as sets, order & multiplicity of words
// doesn't matter)
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

    fn new_indexed_place(title: String, description: String, pos: MapPoint) -> IndexedPlace {
        let id = title.clone();
        IndexedPlace {
            id,
            title,
            description,
            pos,
            ..Default::default()
        }
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

        assert!(is_similar_text(&e1.title, &e2.title, 0.2, 0)); // only 2 characters changed
        assert!(!is_similar_text(&e1.title, &e2.title, 0.1, 0)); // more than one character changed
        assert!(is_similar_text(&e3.title, &e4.title, 0.0, 2)); // only 2 words changed
        assert!(!is_similar_text(&e3.title, &e4.title, 0.0, 1)); // more than 1
                                                                 // word changed
    }
    #[test]
    fn test_is_duplicate_of() {
        let new_x = NewPlace {
            title: "Ein Eintrag Blablabla".into(),
            description: "Hallo! Ein Eintrag".into(),
            lat: 47.23153745093964,
            lng: 5.003816366195679,
            street: None,
            zip: None,
            city: None,
            country: None,
            state: None,
            contact_name: None,
            email: None,
            telephone: None,
            homepage: None,
            opening_hours: None,
            founded_on: None,
            categories: vec![],
            tags: vec![],
            license: "ODbL-1.0".into(),
            image_url: None,
            image_link_url: None,
            custom_links: vec![],
        };
        let new_y = NewPlace {
            lat: 47.13153745093964,
            ..new_x.clone()
        };

        let x = new_indexed_place(
            new_x.title.clone(),
            new_x.description.clone(),
            MapPoint::from_lat_lng_deg(new_x.lat, new_x.lng),
        );
        // small hamming distance: 2 words in title missing
        let similar_title_words1 = new_indexed_place(
            "Eintrag".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093970, 5.003816366195679),
        );
        // small hamming distance: 2 words in title differ
        let similar_title_words2 = new_indexed_place(
            "En Eintrg Blablala".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        // small levenshtein distance: some typos in title
        let similar_title_characters = new_indexed_place(
            "Enn Eintrxg Blablalx".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );

        assert!(is_duplicate_of(&new_x, &x));
        assert!(is_duplicate_of(&new_x, &similar_title_words1));
        assert!(is_duplicate_of(&new_x, &similar_title_words2));
        assert!(is_duplicate_of(&new_x, &similar_title_characters));

        assert!(!is_duplicate_of(&new_y, &x));
    }

    #[test]
    fn test_is_duplicate() {
        let p1 = new_place(
            "Ein Eintrag Blablabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093964, 5.003816366195679),
        );
        let p2 = new_place(
            "Eintrag".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093970, 5.003816366195679),
        );
        let p4 = new_place(
            "En Eintrg Blablala".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );

        let ip2 = new_indexed_place(
            "Eintrag".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093970, 5.003816366195679),
        );
        let ip3 = new_indexed_place(
            "Enn Eintrxg Blablalx".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let ip4 = new_indexed_place(
            "En Eintrg Blablala".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(47.23153745093955, 5.003816366195679),
        );
        let ip5 = new_indexed_place(
            "Ein Eintrag Blabla".to_string(),
            "Hallo! Ein Eintrag".to_string(),
            MapPoint::from_lat_lng_deg(40.23153745093960, 5.003816366195670),
        );

        // titles have a word that is equal
        assert_eq!(Some(DuplicateType::SimilarWords), is_duplicate(&p1, &ip2));
        // titles similar: small levenshtein distance
        assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&p1, &ip4));
        // titles similar: small hamming distance
        assert_eq!(Some(DuplicateType::SimilarChars), is_duplicate(&p1, &ip3));
        // titles not similar
        assert_eq!(None, is_duplicate(&p2, &ip4));
        // places not located close together
        assert_eq!(None, is_duplicate(&p4, &ip5));
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
        assert!(words_equal_except_k_words("ab abc a", "ab abc b", 1));
        assert!(words_equal_except_k_words("ab abc a", "abc ab", 1));
        assert!(words_equal_except_k_words("ab ac a", "abc ab ab", 2));
        assert!(!words_equal_except_k_words("a a a", "ab abc", 2));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(3, levenshtein_distance("012a34c", "0a3c")); // delete 1,2 and 4
        assert_eq!(1, levenshtein_distance("12345", "a12345")); // insert a
        assert_eq!(1, levenshtein_distance("aabaa", "aacaa")); // replace b by c
    }
}
