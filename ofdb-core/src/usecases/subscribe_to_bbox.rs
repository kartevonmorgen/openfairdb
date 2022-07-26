use super::prelude::*;
use crate::{usecases::unsubscribe_all_bboxes, util::validate};

pub fn subscribe_to_bbox<R>(repo: &R, user_email: String, bbox: MapBbox) -> Result<()>
where
    R: SubscriptionRepo + UserRepo,
{
    if !validate::is_valid_bbox(&bbox) {
        return Err(Error::Bbox);
    }

    // TODO: support multiple subscriptions in KVM (frontend)
    // In the meanwhile we just replace existing subscriptions
    // with a new one.
    unsubscribe_all_bboxes(repo, &user_email)?;

    let id = Id::new();
    repo.create_bbox_subscription(&BboxSubscription {
        id,
        user_email,
        bbox,
    })?;
    Ok(())
}
