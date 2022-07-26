use super::prelude::*;

pub fn get_bbox_subscriptions<R>(repo: &R, user_email: &str) -> Result<Vec<BboxSubscription>>
where
    R: SubscriptionRepo,
{
    Ok(repo
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.user_email == user_email)
        .collect())
}
