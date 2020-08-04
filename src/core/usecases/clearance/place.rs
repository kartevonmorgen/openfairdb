use crate::core::prelude::*;

pub(crate) fn add_pending_clearance<R: PlaceClearanceRepo>(
    repo: &R,
    org_ids: &[Id],
    pending_clearance: &PendingClearanceForPlace,
) -> Result<usize> {
    Ok(repo.add_pending_clearance_for_places(org_ids, pending_clearance)?)
}

pub fn count_pending_clearances<R: OrganizationRepo + PlaceClearanceRepo>(
    repo: &R,
    org_token: &str,
) -> Result<u64> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    Ok(repo.count_pending_clearances_for_places(&org.id)?)
}

pub fn list_pending_clearances<R: OrganizationRepo + PlaceClearanceRepo>(
    repo: &R,
    org_token: &str,
    pagination: &Pagination,
) -> Result<Vec<PendingClearanceForPlace>> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    Ok(repo.list_pending_clearances_for_places(&org.id, pagination)?)
}

pub fn update_pending_clearances<R: OrganizationRepo + PlaceClearanceRepo>(
    repo: &R,
    org_token: &str,
    clearances: &[ClearanceForPlace],
) -> Result<usize> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    let count = repo.update_pending_clearances_for_places(&org.id, clearances)?;
    log::info!(
        "Updated {} of {} pending clearance(s) for places on behalf of organization '{}'",
        count,
        clearances.len(),
        org.name
    );
    repo.cleanup_pending_clearances_for_places(&org.id)?;
    Ok(count)
}
