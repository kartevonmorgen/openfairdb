use crate::core::{prelude::*, util::sort::Rated};

use failure::Fallible;

pub fn index_place(
    indexer: &dyn PlaceIndexer,
    place: &Place,
    ratings: &[Rating],
) -> Fallible<AvgRatings> {
    let avg_ratings = place.avg_ratings(ratings);
    indexer.add_or_update_place(place, &avg_ratings)?;
    Ok(avg_ratings)
}

pub fn unindex_place(indexer: &dyn PlaceIndexer, id: &Id) -> Fallible<()> {
    indexer.remove_by_id(id)
}

pub fn index_event(indexer: &dyn EventIndexer, event: &Event) -> Fallible<()> {
    indexer.add_or_update_event(event)
}

pub fn unindex_event(indexer: &dyn EventIndexer, id: &Id) -> Fallible<()> {
    indexer.remove_by_id(id)
}
