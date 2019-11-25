use crate::core::{prelude::*, util::sort::Rated};

use failure::Fallible;

pub fn index_place(
    indexer: &mut dyn PlaceIndexer,
    place: &Place,
    ratings: &[Rating],
) -> Fallible<AvgRatings> {
    let avg_ratings = place.avg_ratings(ratings);
    indexer.add_or_update_place(place, &avg_ratings)?;
    Ok(avg_ratings)
}

pub fn unindex_place(indexer: &mut dyn PlaceIndexer, id: &str) -> Fallible<()> {
    indexer.remove_place_by_id(id)?;
    Ok(())
}
