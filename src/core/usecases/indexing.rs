use crate::core::{prelude::*, util::sort::Rated};

use failure::Fallible;

pub fn index_entry(
    indexer: &mut EntryIndexer,
    entry: &Entry,
    ratings: &[Rating],
) -> Fallible<AvgRatingValue> {
    let avg_rating = entry.avg_rating(ratings);
    indexer.add_or_update_entry(entry, avg_rating)?;
    Ok(avg_rating)
}

pub fn unindex_entry(indexer: &mut EntryIndexer, entry_id: &str) -> Fallible<()> {
    indexer.remove_entry_by_id(entry_id)?;
    Ok(())
}
