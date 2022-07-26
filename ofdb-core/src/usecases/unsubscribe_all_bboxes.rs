use super::prelude::*;

pub fn unsubscribe_all_bboxes<R>(repo: &R, user_email: &str) -> Result<()>
where
    R: SubscriptionRepo,
{
    Ok(repo.delete_bbox_subscriptions_by_email(user_email)?)
}
