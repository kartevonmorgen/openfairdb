use crate::core::prelude::*;

use std::collections::HashMap;

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

pub fn clear_repo_results<R: PlaceRepo + PlaceClearanceRepo>(
    repo: &R,
    org_id: &Id,
    org_tag: &str,
    results: Vec<(Place, ReviewStatus)>,
) -> Result<Vec<(Place, ReviewStatus)>> {
    let place_ids: Vec<_> = results.iter().map(|(p, _)| p.id.as_str()).collect();
    let pending_clearances = repo.load_pending_clearances_for_places(org_id, &place_ids)?;
    if pending_clearances.is_empty() {
        // No filtering required
        return Ok(results);
    }
    let pending_clearances: HashMap<_, _> = pending_clearances
        .into_iter()
        .map(|p| (p.place_id.to_string(), p))
        .collect();
    let mut cleared_results = Vec::with_capacity(results.len());
    for (mut place, mut review_status) in results.into_iter() {
        debug_assert!(place
            .tags
            .iter()
            .map(String::as_str)
            .any(|tag| tag == org_tag));
        let pending_clearance = pending_clearances.get(place.id.as_str());
        if let Some(pending_clearance) = pending_clearance {
            if let Some(last_cleared_revision) = &pending_clearance.last_cleared_revision {
                let (last_cleared_place, last_cleared_status) =
                    repo.load_place_revision(place.id.as_str(), *last_cleared_revision)?;
                debug_assert_eq!(*last_cleared_revision, last_cleared_place.revision);
                let last_cleared_tags = &last_cleared_place.tags;
                if !last_cleared_tags
                    .iter()
                    .map(String::as_str)
                    .any(|tag| tag == org_tag)
                {
                    // Remove previously untagged places from the result
                    continue;
                }
                // Replace the actual/current search result item with the last cleared revision
                place = last_cleared_place;
                review_status = last_cleared_status;
            } else {
                // Skip newly created but not yet cleared entry
                continue;
            }
        }
        cleared_results.push((place, review_status));
    }
    Ok(cleared_results)
}
