use super::*;
use crate::core::error::Error;

#[get("/places/clearance/count")]
pub fn count_pending_clearances(db: sqlite::Connections, auth: Auth) -> Result<json::ResultCount> {
    let db = db.shared()?;
    let count =
        usecases::clearance::place::count_pending_clearances(&db, &auth.organization(&db)?)?;
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
        &db,
        &auth.organization(&db)?,
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
    clearances: JsonResult<Vec<json::ClearanceForPlace>>,
) -> Result<json::ResultCount> {
    let clearances: Vec<_> = clearances?
        .into_inner()
        .into_iter()
        .map(Into::into)
        .collect();
    let org = auth.organization(&db.shared()?)?;
    let count =
        usecases::clearance::place::update_pending_clearances(&db.exclusive()?, &org, &clearances)?;
    Ok(Json(json::ResultCount {
        count: count as u64,
    }))
}

#[post("/places/<ids>/review", data = "<review>")]
pub fn post_review(
    auth: Auth,
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    ids: String,
    review: JsonResult<json::Review>,
) -> Result<()> {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        log::debug!("No places to review");
        return Err(Error::Parameter(ParameterError::EmptyIdList).into());
    }
    let reviewer_email = {
        let db = db.shared()?;
        // Only scouts and admins are entitled to review places
        auth.user_with_min_role(&db, Role::Scout)
            .map_err(|err| {
                log::debug!("Unauthorized user: {}", err);
                err
            })?
            .email
    };
    let json::Review { status, comment } = review
        .map_err(|err| {
            log::debug!("Invalid review: {:?}", err);
            err
        })?
        .into_inner();
    // TODO: Record context information
    let context = None;
    let review = usecases::Review {
        context,
        reviewer_email: reviewer_email.into(),
        status: status.into(),
        comment,
    };
    let update_count =
        flows::review_places(&db, &mut search_engine, &ids, review).map_err(|err| {
            log::debug!("Unable to review places: {}", err);
            err
        })?;
    if update_count < ids.len() {
        log::warn!(
            "Applied review to only {} of {} place(s): {:?}",
            update_count,
            ids.len(),
            ids
        );
    }
    Ok(Json(()))
}
