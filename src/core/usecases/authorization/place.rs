use crate::core::prelude::*;

pub(crate) fn add_pending_authorization<R: PlaceAuthorizationRepo>(
    repo: &R,
    org_ids: &[Id],
    pending_authorization: &PendingAuthorizationForPlace,
) -> Result<usize> {
    Ok(repo.add_pending_authorization_for_place(org_ids, pending_authorization)?)
}

pub fn count_pending_authorizations<R: OrganizationRepo + PlaceAuthorizationRepo>(
    repo: &R,
    org_token: &str,
) -> Result<u64> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    Ok(repo.count_pending_authorizations_for_places(&org.id)?)
}

pub fn list_pending_authorizations<R: OrganizationRepo + PlaceAuthorizationRepo>(
    repo: &R,
    org_token: &str,
    pagination: &Pagination,
) -> Result<Vec<PendingAuthorizationForPlace>> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    Ok(repo.list_pending_authorizations_for_places(&org.id, pagination)?)
}

pub fn acknowledge_pending_authorizations<R: OrganizationRepo + PlaceAuthorizationRepo>(
    repo: &R,
    org_token: &str,
    authorizations: &[AuthorizationForPlace],
) -> Result<usize> {
    let org = repo.get_org_by_api_token(org_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    let acknowledged_count =
        repo.replace_pending_authorizations_for_places(&org.id, authorizations)?;
    log::info!(
        "Acknowledged {} of {} pending authorization(s) for places on behalf of organization '{}'",
        acknowledged_count,
        authorizations.len(),
        org.name
    );
    repo.cleanup_pending_authorizations_for_places(&org.id)?;
    Ok(acknowledged_count)
}
