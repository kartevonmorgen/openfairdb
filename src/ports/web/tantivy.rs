use crate::core::db::{EntryGateway, EntryIndex, EntryIndexQuery, EntryIndexer};
use crate::core::entities::Entry;
use crate::infrastructure::db::tantivy::TantivyEntryIndex;
use crate::infrastructure::error::AppError;

use failure::Fallible;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct SearchEngine(Arc<Mutex<Box<dyn EntryIndexer + Send>>>);

pub fn create_search_engine_in_ram() -> Result<SearchEngine, AppError> {
    let entry_index = TantivyEntryIndex::create_in_ram()
        .map_err(|err| AppError::Other(Box::new(err.compat())))?;
    Ok(SearchEngine(Arc::new(Mutex::new(Box::new(entry_index)))))
}

pub fn create_search_engine<P: AsRef<Path>>(path: Option<P>) -> Result<SearchEngine, AppError> {
    let entry_index =
        TantivyEntryIndex::create(path).map_err(|err| AppError::Other(Box::new(err.compat())))?;
    Ok(SearchEngine(Arc::new(Mutex::new(Box::new(entry_index)))))
}

impl<'a, 'r> FromRequest<'a, 'r> for SearchEngine {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<SearchEngine, ()> {
        let search_engine = request.guard::<State<SearchEngine>>()?;
        Outcome::Success(search_engine.clone())
    }
}

impl EntryIndex for SearchEngine {
    fn query_entries(
        &self,
        entries: &EntryGateway,
        query: &EntryIndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Entry>> {
        let entry_index = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entry_index.query_entries(entries, query, limit)
    }
}

impl EntryIndexer for SearchEngine {
    fn add_or_update_entry(&mut self, entry: &Entry) -> Fallible<()> {
        let mut entry_indexer = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entry_indexer.add_or_update_entry(entry)
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        let mut entry_indexer = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entry_indexer.remove_entry_by_id(id)
    }

    fn flush(&mut self) -> Fallible<()> {
        let mut entry_indexer = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entry_indexer.flush()
    }
}
