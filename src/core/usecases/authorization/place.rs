use crate::core::prelude::*;

pub fn add_pending_authorization<R: PlaceAuthorizationRepo>(
    repo: &R,
    org_ids: &[Id],
    pending_authorization: &PendingAuthorizationForPlace,
) -> Result<usize> {
    Ok(repo.add_pending_authorization_for_place(org_ids, pending_authorization)?)
}

pub fn count_pending_authorizations<R: PlaceAuthorizationRepo>(
    repo: &R,
    org_id: &Id,
) -> Result<u64> {
    Ok(repo.count_pending_authorizations_for_places(org_id)?)
}

pub fn list_pending_authorizations<R: PlaceAuthorizationRepo>(
    repo: &R,
    org_id: &Id,
    pagination: &Pagination,
) -> Result<Vec<PendingAuthorizationForPlace>> {
    Ok(repo.list_pending_authorizations_for_places(org_id, pagination)?)
}

pub fn acknowledge_pending_authorizations<R: PlaceAuthorizationRepo>(
    repo: &R,
    org_id: &Id,
    authorizations: &[AuthorizationForPlace],
) -> Result<usize> {
    repo.replace_pending_authorizations_for_places(org_id, authorizations)?;
    Ok(repo.cleanup_pending_authorizations_for_places(org_id)?)
}
