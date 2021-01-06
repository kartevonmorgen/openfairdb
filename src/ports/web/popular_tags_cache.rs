use crate::{
    core::db::PlaceRepo,
    ports::web::{MostPopularTagsParams, Pagination},
};
use anyhow::Result;
use ofdb_boundary::TagFrequency;

pub struct PopularTagsCache(Vec<TagFrequency>);

pub fn new_from_db<R: PlaceRepo>(db: &R) -> Result<PopularTagsCache> {
    let params = MostPopularTagsParams {
        min_count: None,
        max_count: None,
    };
    let offset = None;
    let limit = None;
    let pagination = Pagination { offset, limit };
    let results = db
        .most_popular_place_revision_tags(&params, &pagination)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(PopularTagsCache(results))
}

impl PopularTagsCache {
    pub fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        let MostPopularTagsParams {
            min_count,
            max_count,
        } = params;
        let Pagination { offset, limit } = pagination;
        let offset = offset.unwrap_or(0) as usize;
        let res = self
            .0
            .iter()
            .filter(|t| {
                if let Some(min) = min_count {
                    t.1 >= *min
                } else {
                    true
                }
            })
            .filter(|t| {
                if let Some(max) = max_count {
                    t.1 <= *max
                } else {
                    true
                }
            })
            .skip(offset);
        let res = if let Some(limit) = limit {
            res.take(*limit as usize).cloned().collect()
        } else {
            res.cloned().collect()
        };
        Ok(res)
    }
}
