use crate::core::prelude::*;

pub struct Review {
    pub context: Option<String>,
    pub reviewer_email: Email,
    pub status: ReviewStatus,
    pub comment: Option<String>,
}

pub fn review_places<D: Db>(db: &D, ids: &[&str], review: Review) -> Result<usize> {
    let Review {
        context,
        reviewer_email,
        status,
        comment,
    } = review;
    let activity = Activity::now(Some(reviewer_email));
    //  TODO: Verify user role
    if status == ReviewStatus::Archived {
        info!(
            "Archiving {} places including ratings and comments",
            ids.len()
        );
        let activity_log = ActivityLog {
            activity,
            context,
            comment,
        };
        let comment_count = db.archive_comments_of_places(ids, &activity_log.activity)?;
        info!(
            "Archived {} comments of {} places",
            comment_count,
            ids.len()
        );
        let rating_count = db.archive_ratings_of_places(ids, &activity_log.activity)?;
        info!("Archived {} ratings of {} places", rating_count, ids.len());
        let place_count = db.review_places(ids, status, &activity_log)?;
        info!(
            "Archived {} places including ratings and comments",
            place_count
        );
        Ok(place_count)
    } else {
        info!(
            "Changing review status of {} places to {}",
            ids.len(),
            ReviewStatusPrimitive::from(status),
        );
        let activity_log = ActivityLog {
            activity,
            context,
            comment,
        };
        let place_count = db.review_places(ids, status, &activity_log)?;
        info!(
            "Changed review status of {} places to {}",
            place_count,
            ReviewStatusPrimitive::from(status)
        );
        Ok(place_count)
    }
}
