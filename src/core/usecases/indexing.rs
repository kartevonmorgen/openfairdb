use crate::core::{prelude::*, util::sort::Rated};

use failure::Fallible;

pub fn index_entry(
    indexer: &mut dyn EntryIndexer,
    place_rev: &PlaceRev,
    ratings: &[Rating],
) -> Fallible<AvgRatings> {
    let avg_ratings = place_rev.avg_ratings(ratings);
    indexer.add_or_update_entry(place_rev, &avg_ratings)?;
    Ok(avg_ratings)
}

pub fn unindex_entry(indexer: &mut dyn EntryIndexer, uid: &str) -> Fallible<()> {
    indexer.remove_entry_by_id(uid)?;
    Ok(())
}
