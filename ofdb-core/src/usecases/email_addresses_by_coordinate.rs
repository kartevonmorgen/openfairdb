use super::prelude::*;
use crate::usecases::bbox_subscriptions_by_coordinate;

pub fn email_addresses_by_coordinate<R>(repo: &R, pos: MapPoint) -> Result<Vec<EmailAddress>>
where
    R: SubscriptionRepo,
{
    Ok(bbox_subscriptions_by_coordinate(repo, pos)?
        .into_iter()
        .map(|s| s.user_email)
        .collect())
}
