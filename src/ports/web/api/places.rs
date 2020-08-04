use super::*;

#[get("/places/clearance/count")]
pub fn count_pending_clearances(
    db: sqlite::Connections,
    org_token: Bearer,
) -> Result<json::ResultCount> {
    let count = usecases::clearance::place::count_pending_clearances(&*db.shared()?, &org_token.0)?;
    Ok(Json(json::ResultCount { count }))
}

#[get("/places/clearance?<offset>&<limit>")]
pub fn list_pending_clearances(
    db: sqlite::Connections,
    org_token: Bearer,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<json::PendingClearanceForPlace>> {
    let pagination = Pagination { offset, limit };
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*db.shared()?,
        &org_token.0,
        &pagination,
    )?;
    Ok(Json(
        pending_clearances.into_iter().map(Into::into).collect(),
    ))
}

#[post("/places/clearance", data = "<clearances>")]
pub fn update_pending_clearances(
    db: sqlite::Connections,
    org_token: Bearer,
    clearances: Json<Vec<json::ClearanceForPlace>>,
) -> Result<json::ResultCount> {
    let clearances: Vec<_> = clearances
        .into_inner()
        .into_iter()
        .map(Into::into)
        .collect();
    let count = usecases::clearance::place::update_pending_clearances(
        &*db.exclusive()?,
        &org_token.0,
        &clearances,
    )?;
    Ok(Json(json::ResultCount {
        count: count as u64,
    }))
}
