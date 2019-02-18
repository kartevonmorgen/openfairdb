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
    pub categories    : Option<Vec<String>>,
    pub text          : String,
    pub tags          : Vec<String>,
    pub entry_ratings : &'a HashMap<String, f64>,
}

pub fn search<D: Db>(db: &D, req: &SearchRequest) -> Result<(Vec<Entry>, Vec<Entry>)> {
    let mut entries = if req.text.is_empty() && req.tags.is_empty() {
        let ext_bbox = filter::extend_bbox(&req.bbox);
        db.get_entries_by_bbox(&ext_bbox)?
    } else {
        db.all_entries()?
    };

    if let Some(ref cat_ids) = req.categories {
        entries = entries
            .into_iter()
            .filter(filter::entries_by_category_ids(cat_ids))
            .collect();
    }

    let mut entries: Vec<_> = entries
        .into_iter()
        .filter(&*filter::entries_by_tags_or_search_text(
            &req.text, &req.tags,
        ))
        .collect();

    entries.sort_by_avg_rating(req.entry_ratings);

    let visible_results: Vec<_> = entries
        .iter()
        .filter(|x| x.in_bbox(&req.bbox))
        .cloned()
        .collect();

    let invisible_results = entries
        .into_iter()
        .filter(|x| !x.in_bbox(&req.bbox))
        .take(MAX_INVISIBLE_RESULTS)
        .collect();

    Ok((visible_results, invisible_results))
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
            categories: None,
            text: "".into(),
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&mut db, &req).unwrap());
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
            categories: None,
            text: "".into(),
            tags: vec![],
            entry_ratings: &entry_ratings,
        };

        b.iter(|| super::search(&mut db, &req).unwrap());
    }

}
