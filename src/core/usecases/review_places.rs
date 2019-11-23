use crate::core::prelude::*;

pub fn review_places<D: Db>(
    db: &D,
    uids: &[&str],
    review_status: ReviewStatus,
    reviewer_email: &str,
) -> Result<usize> {
    let activity = Activity::now(Some(reviewer_email.into()));
    //  TODO: Verify user role
    if review_status == ReviewStatus::Archived {
        info!(
            "Archiving {} places including ratings and comments",
            uids.len()
        );
        let activity_log = ActivityLog {
            activity,
            context: None,
            memo: Some("archived".into()),
        };
        let comment_count = db.archive_comments_of_places(uids, &activity_log.activity)?;
        info!(
            "Archived {} comments of {} places",
            comment_count,
            uids.len()
        );
        let rating_count = db.archive_ratings_of_places(uids, &activity_log.activity)?;
        info!("Archived {} ratings of {} places", rating_count, uids.len());
        let place_count = db.review_places(uids, review_status, &activity_log)?;
        info!(
            "Archived {} places including ratings and comments",
            place_count
        );
        Ok(place_count)
    } else {
        info!(
            "Changing review status of {} places to {}",
            uids.len(),
            ReviewStatusPrimitive::from(review_status),
        );
        let activity_log = ActivityLog {
            activity,
            context: None,
            memo: Some("status changed".into()),
        };
        let place_count = db.review_places(uids, review_status, &activity_log)?;
        info!(
            "Changed review status of {} places to {}",
            place_count,
            ReviewStatusPrimitive::from(review_status)
        );
        Ok(place_count)
    }
}
