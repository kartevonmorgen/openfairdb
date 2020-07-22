use super::*;

#[get("/places/pending-authorizations/count")]
pub fn count_pending_authorizations(
    db: sqlite::Connections,
    org_token: Bearer,
) -> Result<json::ResultCount> {
    let count =
        usecases::authorization::place::count_pending_authorizations(&*db.shared()?, &org_token.0)?;
    Ok(Json(json::ResultCount { count }))
}

#[get("/places/pending-authorizations?<offset>&<limit>")]
pub fn list_pending_authorizations(
    db: sqlite::Connections,
    org_token: Bearer,
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<json::PendingAuthorizationForPlace>> {
    let pagination = Pagination { offset, limit };
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*db.shared()?,
        &org_token.0,
        &pagination,
    )?;
    Ok(Json(
        pending_authorizations.into_iter().map(Into::into).collect(),
    ))
}

#[post(
    "/places/pending-authorizations/acknowledge",
    data = "<authorizations>"
)]
pub fn acknowledge_pending_authorizations(
    db: sqlite::Connections,
    org_token: Bearer,
    authorizations: Json<Vec<json::AuthorizationForPlace>>,
) -> Result<json::ResultCount> {
    let authorizations: Vec<_> = authorizations
        .into_inner()
        .into_iter()
        .map(Into::into)
        .collect();
    let count = usecases::authorization::place::acknowledge_pending_authorizations(
        &*db.exclusive()?,
        &org_token.0,
        &authorizations,
    )?;
    Ok(Json(json::ResultCount {
        count: count as u64,
    }))
}
