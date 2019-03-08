use crate::core::prelude::*;
use crate::core::util::{filter, geo::MapBbox};

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a, 'b> {
    pub bbox          : MapBbox,
    pub categories    : Vec<&'a str>,
    pub ids           : Vec<&'b str>,
    pub tags          : Vec<String>,
    pub text          : Option<String>,
}

pub fn search(
    index: &EntryIndex,
    req: SearchRequest,
    limit: usize,
) -> Result<(Vec<IndexedEntry>, Vec<IndexedEntry>)> {
    let visible_bbox: MapBbox = req.bbox;

    let index_query = EntryIndexQuery {
        bbox: Some(filter::extend_bbox(&visible_bbox)),
        categories: req.categories,
        ids: req.ids,
        tags: req.tags,
        text: req.text,
    };

    let entries = index
        .query_entries(&index_query, limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;

    let (visible_entries, invisible_entries): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|e| visible_bbox.contains_point(&e.pos));

    Ok((visible_entries, invisible_entries))
}

/// The global search usecase is like the one
/// of usual internet search engines that exists
/// of only one single search input.
/// So here we don't care about tags, categories etc.
/// We also ignore the rating of an entry for now.
pub fn global_search(index: &EntryIndex, txt: &str, limit: usize) -> Result<Vec<IndexedEntry>> {
    let index_query = EntryIndexQuery {
        text: Some(txt.into()),
        ..Default::default()
    };

    let entries = index
        .query_entries(&index_query, limit)
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
        let mut db = MockDb::default();
        let (entries, ratings) = sort::tests::create_entries_with_ratings(1_000);
        db.entries = entries.into();
        db.ratings = ratings.into();
        let req = SearchRequest {
            bbox: MapBbox::new(
                MapPoint::from_lat_lng_deg(-10.0, -10.0),
                MapPoint::from_lat_lng_deg(10.0, 10.0),
            ),
            categories: vec![],
            ids: vec![],
            tags: vec![],
            text: None,
        };

        b.iter(|| super::search(&db, req.clone(), 100).unwrap());
    }

    #[ignore]
    #[bench]
    fn bench_search_in_10_000_rated_entries(b: &mut Bencher) {
        let mut db = MockDb::default();
        let (entries, ratings) = sort::tests::create_entries_with_ratings(10_000);
        db.entries = entries.into();
        db.ratings = ratings.into();
        let req = SearchRequest {
            bbox: MapBbox::new(
                MapPoint::from_lat_lng_deg(-10.0, -10.0),
                MapPoint::from_lat_lng_deg(10.0, 10.0),
            ),
            categories: vec![],
            ids: vec![],
            tags: vec![],
            text: None,
        };

        b.iter(|| super::search(&db, req.clone(), 100).unwrap());
    }

}
