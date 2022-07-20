use super::{clearance, prelude::*};

pub fn load_places<R: PlaceRepo + PlaceClearanceRepo + OrganizationRepo>(
    repo: &R,
    ids: &[&str],
    org_tag: Option<&str>,
) -> Result<Vec<(Place, ReviewStatus)>> {
    let places = repo.get_places(ids)?;
    if let Some(org_tag) = org_tag {
        if let Some(org_id) = repo.map_tag_to_clearance_org_id(org_tag)? {
            return clearance::place::clear_repo_results(repo, &org_id, org_tag, places);
        }
    }
    Ok(places)
}
