use super::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Review {
    pub context: Option<String>,
    pub reviewer_email: EmailAddress,
    pub status: ReviewStatus,
    pub comment: Option<String>,
}

pub fn review_places<R>(repo: &R, ids: &[&str], review: Review) -> Result<usize>
where
    R: PlaceRepo,
{
    let Review {
        context,
        reviewer_email,
        status,
        comment,
    } = review;
    let activity = Activity::now(Some(reviewer_email));
    //  TODO: Verify user role here instead of in web api
    log::info!(
        "Changing review status of {} places to {}",
        ids.len(),
        ReviewStatusPrimitive::from(status),
    );
    let activity_log = ActivityLog {
        activity,
        context,
        comment,
    };
    let place_count = repo.review_places(ids, status, &activity_log)?;
    log::info!(
        "Changed review status of {} places to {}",
        place_count,
        ReviewStatusPrimitive::from(status)
    );
    Ok(place_count)
}
