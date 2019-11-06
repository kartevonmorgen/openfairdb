use crate::core::prelude::*;

pub fn archive_places<D: Db>(db: &D, ids: &[&str], email: &str) -> Result<usize> {
    info!(
        "Archiving {} places including ratings and comments",
        ids.len()
    );
    let activity = UserActivity::now(email.into());
    let comment_count = db.archive_comments_of_places(ids, &activity)?;
    info!(
        "Archived {} comments of {} places",
        comment_count,
        ids.len()
    );
    let rating_count = db.archive_ratings_of_places(ids, &activity)?;
    info!("Archived {} ratings of {} places", rating_count, ids.len());
    let place_count = db.change_status_of_places(ids, Status::archived(), &activity)?;
    info!(
        "Archived {} places including ratings and comments",
        place_count
    );
    Ok(place_count)
}
