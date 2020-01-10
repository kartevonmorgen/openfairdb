use crate::core::{prelude::*, util::sort::Rated};

use anyhow::Result as Fallible;

pub fn reindex_place(
    indexer: &dyn PlaceIndexer,
    place: &Place,
    status: ReviewStatus,
    ratings: &[Rating],
) -> Fallible<AvgRatings> {
    let avg_ratings = place.avg_ratings(ratings);
    indexer.add_or_update_place(place, status, &avg_ratings)?;
    Ok(avg_ratings)
}

pub fn index_event(indexer: &dyn EventIndexer, event: &Event) -> Fallible<()> {
    indexer.add_or_update_event(event)
}

pub fn unindex_event(indexer: &dyn EventIndexer, id: &Id) -> Fallible<()> {
    indexer.remove_by_id(id)
}
