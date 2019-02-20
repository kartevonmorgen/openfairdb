use crate::infrastructure::error::AppError;
use crate::infrastructure::db::tantivy::TantivyEntryIndex;
use crate::core::db::{EntryIndex, EntryIndexQuery, EntryIndexer, EntryGateway};
use crate::core::entities::Entry;

use failure::Fallible;
use std::sync::{Arc, Mutex};
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

#[derive(Clone)]
pub struct SearchEngine(Arc<Mutex<Box<dyn EntryIndexer + Send>>>);

pub fn create_search_engine() -> Result<SearchEngine, AppError> {
    let entry_index = TantivyEntryIndex::create().map_err(|err| AppError::Other(Box::new(err.compat())))?;
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
    fn query_entries(&self, entries: &EntryGateway, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<Entry>> {
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
