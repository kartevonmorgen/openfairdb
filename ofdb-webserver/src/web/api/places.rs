use super::*;

#[get("/places/<id>")]
pub fn get_place(
    db: sqlite::Connections,
    id: String,
) -> Result<(json::PlaceRoot, json::PlaceRevision, json::ReviewStatus)> {
    let (place, status) = {
        let db = db.shared()?;
        db.get_place(&id)?
    };
    let (place_root, place_revision) = place.into();
    Ok(Json((
        place_root.into(),
        place_revision.into(),
        status.into(),
    )))
}

#[get("/places/<id>/history/<revision>")]
pub fn get_place_history_revision(
    db: sqlite::Connections,
    auth: Auth,
    id: String,
    revision: RevisionValue,
) -> Result<json::PlaceHistory> {
    let place_history = {
        let db = db.shared()?;

        // The history contains e-mail addresses of registered users
        // is only permitted for scouts and admins or organizations!
        if auth.user_with_min_role(&db, Role::Scout).is_err() {
            auth.organization(&db)?;
        }

        db.get_place_history(&id, Some(revision.into()))?
    };
    Ok(Json(place_history.into()))
}

#[get("/places/<id>/history", rank = 2)]
pub fn get_place_history(
    db: sqlite::Connections,
    auth: Auth,
    id: String,
) -> Result<json::PlaceHistory> {
    let place_history = {
        let db = db.shared()?;

        // The history contains e-mail addresses of registered users
        // is only permitted for scouts and admins or for organizations!
        if auth.user_with_min_role(&db, Role::Scout).is_err() {
            auth.organization(&db)?;
        }

        db.get_place_history(&id, None)?
    };
    Ok(Json(place_history.into()))
}

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
    let ids = crate::core::util::split_ids(&ids);
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
        reviewer_email,
        status: status.into(),
        comment,
    };
    let update_count =
        flows::review_places(&db, &mut *search_engine, &ids, review).map_err(|err| {
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

#[derive(Deserialize)]
pub struct ReviewWithToken {
    pub token: String,
    pub status: json::ReviewStatus,
}

#[post("/places/review-with-token", data = "<review>")]
pub fn post_review_with_token(
    connections: sqlite::Connections,
    review: JsonResult<ReviewWithToken>,
) -> Result<()> {
    let ReviewWithToken { token, status } = review
        .map_err(|err| {
            log::debug!("Invalid review: {:?}", err);
            err
        })?
        .into_inner();
    let review_nonce = ReviewNonce::decode_from_str(&token)?;
    let now = Timestamp::now();
    flows::review_place_with_review_nonce(&connections, review_nonce, status.into(), now)?;
    Ok(Json(()))
}

// FIXME: Limit the total number of not updated places
// to avoid cloning the whole database!!
#[get("/places/not-updated?<since>&<offset>&<limit>")]
pub fn get_not_updated(
    auth: Auth,
    db: sqlite::Connections,
    since: i64, // in seconds
    offset: Option<u64>,
    limit: Option<u64>,
) -> Result<Vec<(json::PlaceRoot, json::PlaceRevision, json::ReviewStatus)>> {
    {
        let db = db.shared()?;
        // Only scouts and admins are entitled to review places
        auth.user_with_min_role(&db, Role::Scout).map_err(|err| {
            log::debug!("Unauthorized user: {}", err);
            err
        })?;
    }
    let pagination = Pagination { offset, limit };
    let entries = {
        let db = db.shared()?;
        db.find_places_not_updated_since(Timestamp::from_secs(since), &pagination)?
    };
    let entries = entries
        .into_iter()
        .map(|(place, status)| {
            let (place_root, place_revision) = place.into();
            (place_root.into(), place_revision.into(), status.into())
        })
        .collect::<Vec<_>>();
    Ok(Json(entries))
}
