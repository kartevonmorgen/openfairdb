use crate::core::prelude::*;
use crate::core::util::filter;

type Result<T> = std::result::Result<T, RepoError>;

pub struct DbEntryIndex<'a, D: Db> {
    db: &'a D,
}

impl<'a, D: Db> DbEntryIndex<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self { db }
    }
}

impl<'a, D: Db> EntryIndex for DbEntryIndex<'a, D> {
    fn add_or_update_entry(&mut self, entry: &Entry) -> Result<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Ok(entry), self.db.get_entry(&entry.id).as_ref());
        debug_assert!(entry == &self.db.get_entry(&entry.id).unwrap());
        Ok(())
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Result<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Err(RepoError::NotFound), self.db.get_entry(&id));
        debug_assert!(self.db.get_entry(&id).is_err());
        Ok(())
    }

    fn query_entries(&self, query: &EntryIndexQuery) -> Result<Vec<Entry>> {
        let mut entries = if let Some(ref bbox) = query.bbox {
            self.db.get_entries_by_bbox(bbox)?
        } else {
            self.db.all_entries()?
        };

        if !query.categories.is_empty() {
            entries = entries
                .into_iter()
                .filter(filter::entries_by_category_ids(&query.categories))
                .collect();
        }

        entries = entries
            .into_iter()
            .filter(&*filter::entries_by_tags_or_search_text(
                query.text.as_ref().map(String::as_str).unwrap_or(""), &query.tags,
            ))
            .collect();

        Ok(entries)
    }
}
