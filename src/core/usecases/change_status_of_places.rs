use crate::core::prelude::*;

pub fn change_status_of_places<D: Db>(
    db: &D,
    ids: &[&str],
    status: Status,
    email: &str,
) -> Result<usize> {
    let activity = Activity::now(Some(email.into()));
    //  TODO: Verify user role
    if status == Status::archived() {
        info!(
            "Archiving {} places including ratings and comments",
            ids.len()
        );
        let activity_log = ActivityLog {
            activity,
            context: None,
            notes: Some("archived".into()),
        };
        let comment_count = db.archive_comments_of_places(ids, &activity_log.activity)?;
        info!(
            "Archived {} comments of {} places",
            comment_count,
            ids.len()
        );
        let rating_count = db.archive_ratings_of_places(ids, &activity_log.activity)?;
        info!("Archived {} ratings of {} places", rating_count, ids.len());
        let place_count = db.change_status_of_places(ids, status, &activity_log)?;
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
        let activity_log = ActivityLog {
            activity,
            context: None,
            notes: Some("status changed".into()),
        };
        let place_count = db.change_status_of_places(ids, status, &activity_log)?;
        info!(
            "Changed status of {} places to {}",
            place_count,
            status.into_inner()
        );
        Ok(place_count)
    }
}
