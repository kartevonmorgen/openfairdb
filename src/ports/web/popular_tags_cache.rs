use crate::{
    core::db::PlaceRepo,
    ports::web::{MostPopularTagsParams, Pagination},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use ofdb_boundary::TagFrequency;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct PopularTagsCache(RwLock<Cache>);

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
    let cache = Cache {
        created_at: Utc::now(),
        data: results,
    };
    Ok(PopularTagsCache(RwLock::new(cache)))
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
        let cached_data = &self.read().data;
        let res = cached_data
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

    pub fn age_in_seconds(&self) -> u64 {
        (Utc::now() - self.read().created_at).num_seconds() as u64
    }

    pub fn update_cache(&self, data: Vec<TagFrequency>) {
        let mut cache = self.write();
        cache.created_at = Utc::now();
        cache.data = data;
    }

    fn read(&self) -> RwLockReadGuard<Cache> {
        match self.0.read() {
            Ok(guard) => guard,
            Err(poison_err) => {
                log::error!("A poisoned RwLockReadGuard for the PopularTagsCache was found.");
                poison_err.into_inner()
            }
        }
    }

    fn write(&self) -> RwLockWriteGuard<Cache> {
        match self.0.write() {
            Ok(guard) => guard,
            Err(poison_err) => {
                log::error!("A poisoned RwLockWriteGuard for the PopularTagsCache was found.");
                poison_err.into_inner()
            }
        }
    }
}

struct Cache {
    created_at: DateTime<Utc>,
    data: Vec<TagFrequency>,
}
