use crate::core::prelude::*;
use crate::core::util::{
    filter::{self, InBBox},
    sort::SortByAverageRating,
};

use std::collections::HashMap;

const MAX_INVISIBLE_RESULTS: usize = 5;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a> {
    pub bbox          : Bbox,
    pub categories    : Vec<String>,
    pub text          : Option<String>,
    pub tags          : Vec<String>,
    pub entry_ratings : &'a HashMap<String, f64>,
}

pub fn search(index: &EntryIndex, req: SearchRequest) -> Result<(Vec<Entry>, Vec<Entry>)> {
    let visible_bbox = req.bbox;

    let index_bbox = if req.text.as_ref().map(String::is_empty).unwrap_or(true) && req.tags.is_empty() {
        Some(filter::extend_bbox(&visible_bbox))
    } else {
        None
    };

    let index_query = EntryIndexQuery {
        bbox: index_bbox,
        text: req.text,
        categories: req.categories,
        tags: req.tags,
    };

    let mut entries = index.query_entries(&index_query)?;

    entries.sort_by_avg_rating(req.entry_ratings);

    let visible_results: Vec<_> = entries
        .iter()
        .filter(|x| x.in_bbox(&visible_bbox))
        .cloned()
        .collect();

    let invisible_results = entries
        .into_iter()
        .filter(|x| !x.in_bbox(&visible_bbox))
        .take(MAX_INVISIBLE_RESULTS)
        .collect();

    Ok((visible_results, invisible_results))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
    use crate::core::usecases::index::DbEntryIndex;
    use crate::core::util::sort;
    use crate::test::Bencher;

    #[bench]
    fn bench_search_in_1_000_rated_entries(b: &mut Bencher) {
        let mut db = MockDb::new();
        let (entries, ratings) = sort::tests::create_entries_with_ratings(1_000);
        db.entries = entries;
        db.ratings = ratings;
        let entry_ratings = HashMap::new();
        let req = SearchRequest {
            bbox: Bbox {
                south_west: Coordinate {
                    lat: -10.0,
                    lng: -10.0,
                },
                north_east: Coordinate {
                    lat: 10.0,
                    lng: 10.0,
                },
            },
            categories: vec![],
            text: None,
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&DbEntryIndex::new(&db), req.clone()).unwrap());
    }

    #[ignore]
    #[bench]
    fn bench_search_in_10_000_rated_entries(b: &mut Bencher) {
        let mut db = MockDb::new();
        let (entries, ratings) = sort::tests::create_entries_with_ratings(10_000);
        db.entries = entries;
        db.ratings = ratings;
        let entry_ratings = HashMap::new();
        let req = SearchRequest {
            bbox: Bbox {
                south_west: Coordinate {
                    lat: -10.0,
                    lng: -10.0,
                },
                north_east: Coordinate {
                    lat: 10.0,
                    lng: 10.0,
                },
            },
            categories: vec![],
            text: None,
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&DbEntryIndex::new(&db), req.clone()).unwrap());
    }

}
