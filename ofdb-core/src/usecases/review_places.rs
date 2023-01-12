use super::prelude::*;
use crate::RepoError;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum ReviewPlaceWithNonceError {
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error("Invalid or outdated place revision")]
    PlaceRevision,
}

pub fn review_place_with_nonce<R>(
    repo: &R,
    review_nonce: ReviewNonce,
    new_status: ReviewStatus,
) -> std::result::Result<(), ReviewPlaceWithNonceError>
where
    R: PlaceRepo,
{
    let ReviewNonce {
        place_id,
        place_revision,
        ..
    } = review_nonce;
    let (place, old_status) = repo.get_place(place_id.as_str())?;

    if place.revision != place_revision {
        return Err(ReviewPlaceWithNonceError::PlaceRevision);
    }

    let activity = Activity::now(None);
    log::info!("Changing review status of place {place_id} (rev: {place_revision:?}) from {old_status:?} to {new_status:?}",);
    let comment = None;
    let context = Some("Reviewed with review token".to_string());
    let activity_log = ActivityLog {
        activity,
        context,
        comment,
    };
    let place_count = repo.review_places(&[place_id.as_str()], new_status, &activity_log)?;
    debug_assert_eq!(place_count, 1);
    log::info!("Changed review status of {place_id} places to {new_status:?}",);
    Ok(())
}
