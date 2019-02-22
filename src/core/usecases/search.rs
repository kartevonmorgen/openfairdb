use crate::core::prelude::*;
use crate::core::util::{
    filter::{self, InBBox},
    geo::MapBbox,
    sort::SortByAverageRating,
};

use std::collections::HashMap;

const MAX_INVISIBLE_RESULTS: usize = 5;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a> {
    pub bbox          : MapBbox,
    pub categories    : Vec<String>,
    pub text          : Option<String>,
    pub tags          : Vec<String>,
    pub entry_ratings : &'a HashMap<String, f64>,
}

pub fn search(
    index: &EntryIndex,
    entries: &EntryGateway,
    req: SearchRequest,
    limit: Option<usize>,
) -> Result<(Vec<Entry>, Vec<Entry>)> {
    let visible_bbox: MapBbox = req.bbox;

    let index_bbox =
        if req.text.as_ref().map(String::is_empty).unwrap_or(true) && req.tags.is_empty() {
            Some(filter::extend_bbox(&visible_bbox))
        } else {
            None
        };

    let index_query = EntryIndexQuery {
        bbox: index_bbox.map(Into::into),
        text: req.text,
        categories: req.categories,
        tags: req.tags,
    };

    // TODO: Avoid to query for more entries than requested once
    // the index sorts results by their avg. rating. Until then
    // we need to request more results than needed, sort them by,
    // their avg. rating and cut off the tail.
    let query_limit = entries.count_entries()? + 100;
    let mut entries = index
        .query_entries(entries, &index_query, query_limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;
    entries.sort_by_avg_rating(req.entry_ratings);
    // TODO: Truncation becomes obsolete if query_limit == limit
    if let Some(limit) = limit {
        entries.truncate(limit);
    }

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

/// The global search usecase is like the one
/// of usual internet search engines that exists
/// of only one single search input.
/// So here we don't care about tags, categories etc.
/// We also ignore the rating of an entry for now.
pub fn global_search(
    index: &EntryIndex,
    entries: &EntryGateway,
    txt: &str,
    limit: usize,
) -> Result<Vec<Entry>> {
    let index_query = EntryIndexQuery {
        bbox: None,
        text: Some(txt.into()),
        categories: vec![],
        tags: vec![],
    };

    let entries = index
        .query_entries(entries, &index_query, limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;

    Ok(entries)
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
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
            bbox: MapBbox::new(
                MapPoint::from_lat_lng_deg(-10.0, -10.0),
                MapPoint::from_lat_lng_deg(10.0, 10.0),
            ),
            categories: vec![],
            text: None,
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&db, &db, req.clone(), Some(100)).unwrap());
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
            bbox: MapBbox::new(
                MapPoint::from_lat_lng_deg(-10.0, -10.0),
                MapPoint::from_lat_lng_deg(10.0, 10.0),
            ),
            categories: vec![],
            text: None,
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&db, &db, req.clone(), Some(100)).unwrap());
    }

}
