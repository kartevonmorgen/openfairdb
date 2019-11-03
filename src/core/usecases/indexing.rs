use crate::core::{prelude::*, util::sort::Rated};

use failure::Fallible;

pub fn index_entry(
    indexer: &mut dyn PlaceIndexer,
    place_rev: &Place,
    ratings: &[Rating],
) -> Fallible<AvgRatings> {
    let avg_ratings = place_rev.avg_ratings(ratings);
    indexer.add_or_update_place(place_rev, &avg_ratings)?;
    Ok(avg_ratings)
}

pub fn unindex_entry(indexer: &mut dyn PlaceIndexer, uid: &str) -> Fallible<()> {
    indexer.remove_place_by_uid(uid)?;
    Ok(())
}
