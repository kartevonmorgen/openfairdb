use super::prelude::*;

pub fn bbox_subscriptions_by_coordinate<R>(repo: &R, pos: MapPoint) -> Result<Vec<BboxSubscription>>
where
    R: SubscriptionRepo,
{
    Ok(repo
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.bbox.contains_point(pos))
        .collect())
}
