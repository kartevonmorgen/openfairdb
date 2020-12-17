use super::*;

#[get("/places/clearance/count")]
pub fn count_pending_clearances(db: sqlite::Connections, auth: Auth) -> Result<json::ResultCount> {
    let db = db.shared()?;
    let count =
        usecases::clearance::place::count_pending_clearances(&*db, &auth.organization(&*db)?)?;
    Ok(Json(json::ResultCount { count }))
}

#[get("/places/clearance?<offset>&<limit>")]
pub fn list_pending_clearances(
    db: sqlite::Connections,
    auth: Auth,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<json::PendingClearanceForPlace>> {
    let pagination = Pagination { offset, limit };
    let db = db.shared()?;
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*db,
        &auth.organization(&*db)?,
        &pagination,
    )?;
    Ok(Json(
        pending_clearances.into_iter().map(Into::into).collect(),
    ))
}

#[post("/places/clearance", data = "<clearances>")]
pub fn update_pending_clearances(
    db: sqlite::Connections,
    auth: Auth,
    clearances: Json<Vec<json::ClearanceForPlace>>,
) -> Result<json::ResultCount> {
    let clearances: Vec<_> = clearances
        .into_inner()
        .into_iter()
        .map(Into::into)
        .collect();
    let count = usecases::clearance::place::update_pending_clearances(
        &*db.exclusive()?,
        &auth.organization(&*db.shared()?)?,
        &clearances,
    )?;
    Ok(Json(json::ResultCount {
        count: count as u64,
    }))
}
