use crate::core::prelude::*;

// Checks if the addition and removal of tags is permitted.
//
// Returns a list with the ids of other organizations that require
// clearance of the pending changes.
//
// If an organization is provided than this organization is excluded
// from both the checks and the pending clearance list.
pub fn authorize_editing<R: OrganizationRepo>(
    repo: &R,
    old_tags: &[String],
    new_tags: &[String],
    org: Option<&Organization>,
) -> Result<Vec<Id>> {
    let org_id = org.map(|org| &org.id);
    let moderated_tags_by_org = repo.get_moderated_tags_by_org(org_id)?;
    ofdb_core::tag::moderated::authorize_editing(moderated_tags_by_org, old_tags, new_tags)
        .map_err(|_| ParameterError::ModeratedTag.into())
}
