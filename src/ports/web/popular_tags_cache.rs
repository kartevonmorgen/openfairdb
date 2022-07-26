use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use ofdb_boundary::TagFrequency;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use time::OffsetDateTime;

use crate::{
    core::repositories::PlaceRepo,
    ports::web::{sqlite, MostPopularTagsParams, Pagination},
};

type Request = (MostPopularTagsParams, Pagination);
type Cache = HashMap<Request, (OffsetDateTime, Vec<TagFrequency>)>;

pub struct PopularTagsCache(RwLock<Cache>);

impl PopularTagsCache {
    pub fn new_from_db<R: PlaceRepo>(db: &R) -> Result<PopularTagsCache> {
        let params = MostPopularTagsParams::default();
        let pagination = Pagination::default();
        let cache = Self(RwLock::new(HashMap::new()));
        let _ = cache.query_and_update(db, &params, &pagination)?;
        Ok(cache)
    }

    pub fn most_popular_place_revision_tags(
        &self,
        db: &sqlite::Connections,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
        max_cache_age: Duration,
    ) -> Result<Vec<TagFrequency>> {
        let cached_results = self.read().get(&(*params, *pagination)).cloned();
        if let Some((created_at, data)) = cached_results {
            let age = Duration::try_from(OffsetDateTime::now_utc() - created_at)?;
            if age < max_cache_age {
                return Ok(data);
            }
        }
        self.query_and_update(&db.shared()?, params, pagination)
    }

    fn query_and_update<R: PlaceRepo>(
        &self,
        db: &R,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        let results = db
            .most_popular_place_revision_tags(params, pagination)?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        let mut cache = self.write();
        cache.insert(
            (*params, *pagination),
            (OffsetDateTime::now_utc(), results.clone()),
        );
        Ok(results)
    }

    fn read(&self) -> RwLockReadGuard<Cache> {
        self.0.read()
    }

    fn write(&self) -> RwLockWriteGuard<Cache> {
        self.0.write()
    }
}
