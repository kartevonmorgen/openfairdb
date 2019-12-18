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
    //  TODO: Verify user role here instead of in web api
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
