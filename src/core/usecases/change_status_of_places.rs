use crate::core::prelude::*;

pub fn change_status_of_places<D: Db>(
    db: &D,
    ids: &[&str],
    status: Status,
    email: &str,
) -> Result<usize> {
    let activity = UserActivity::now(email.into());
    //  TODO: Verify user role
    if status == Status::archived() {
        let activity = UserActivity::now(email.into());
        info!(
            "Archiving {} places including ratings and comments",
            ids.len()
        );
        let comment_count = db.archive_comments_of_places(ids, &activity)?;
        info!(
            "Archived {} comments of {} places",
            comment_count,
            ids.len()
        );
        let rating_count = db.archive_ratings_of_places(ids, &activity)?;
        info!("Archived {} ratings of {} places", rating_count, ids.len());
        let place_count = db.change_status_of_places(ids, status, &activity)?;
        info!(
            "Archived {} places including ratings and comments",
            place_count
        );
        Ok(place_count)
    } else {
        info!(
            "Changing status of {} places to {}",
            ids.len(),
            status.into_inner()
        );
        let place_count = db.change_status_of_places(ids, status, &activity)?;
        info!(
            "Changed status of {} places to {}",
            place_count,
            status.into_inner()
        );
        Ok(place_count)
    }
}
