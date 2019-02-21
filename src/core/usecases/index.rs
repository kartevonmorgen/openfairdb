use crate::core::prelude::*;
use crate::core::util::filter;

use failure::{format_err, Fallible};

impl<D> EntryIndexer for D
where
    D: Db,
{
    fn add_or_update_entry(&mut self, entry: &Entry) -> Fallible<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Ok(entry), self.db.get_entry(&entry.id).as_ref());
        debug_assert!(entry == &self.get_entry(&entry.id).unwrap());
        Ok(())
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Err(RepoError::NotFound), self.db.get_entry(&id));
        debug_assert!(self.get_entry(&id).is_err());
        Ok(())
    }

    fn flush(&mut self) -> Fallible<()> {
        Ok(())
    }
}

impl<D> EntryIndex for D
where
    D: Db,
{
    fn query_entries(
        &self,
        _entries: &EntryGateway,
        query: &EntryIndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Entry>> {
        let mut entries = if let Some(ref bbox) = query.bbox {
            let bbox = Bbox {
                south_west: Coordinate {
                    lat: bbox.south_west().lat().to_deg(),
                    lng: bbox.south_west().lng().to_deg(),
                },
                north_east: Coordinate {
                    lat: bbox.north_east().lat().to_deg(),
                    lng: bbox.north_east().lng().to_deg(),
                },
            };
            self.get_entries_by_bbox(&bbox)
        } else {
            self.all_entries()
        }
        .map_err(|err| format_err!("{}", err))?;

        if !query.categories.is_empty() {
            entries = entries
                .into_iter()
                .filter(filter::entries_by_category_ids(&query.categories))
                .collect();
        }

        entries = entries
            .into_iter()
            .take(limit)
            .filter(&*filter::entries_by_tags_or_search_text(
                query.text.as_ref().map(String::as_str).unwrap_or(""),
                &query.tags,
            ))
            .collect();

        Ok(entries)
    }
}
