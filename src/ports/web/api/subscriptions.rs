use super::*;
use crate::core::error::Error;

#[post(
    "/subscribe-to-bbox",
    format = "application/json",
    data = "<coordinates>"
)]
pub fn subscribe_to_bbox(
    db: sqlite::Connections,
    auth: Auth,
    coordinates: JsonResult<Vec<json::Coordinate>>,
) -> Result<()> {
    let email = auth.account_email()?;
    let sw_ne: Vec<_> = coordinates?
        .into_inner()
        .into_iter()
        .map(MapPoint::from)
        .collect();
    if sw_ne.len() != 2 {
        return Err(Error::Parameter(ParameterError::Bbox).into());
    }
    let bbox = geo::MapBbox::new(sw_ne[0], sw_ne[1]);
    usecases::subscribe_to_bbox(&db.exclusive()?, email.to_string(), bbox)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
pub fn unsubscribe_all_bboxes(db: sqlite::Connections, auth: Auth) -> Result<()> {
    let email = auth.account_email()?;
    usecases::unsubscribe_all_bboxes(&db.exclusive()?, email)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
pub fn get_bbox_subscriptions(
    db: sqlite::Connections,
    account: Account,
) -> Result<Vec<json::BboxSubscription>> {
    let email = account.email();
    let user_subscriptions = usecases::get_bbox_subscriptions(&db.shared()?, email)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.id.into(),
            south_west_lat: s.bbox.southwest().lat().to_deg(),
            south_west_lng: s.bbox.southwest().lng().to_deg(),
            north_east_lat: s.bbox.northeast().lat().to_deg(),
            north_east_lng: s.bbox.northeast().lng().to_deg(),
        })
        .collect();
    Ok(Json(user_subscriptions))
}
