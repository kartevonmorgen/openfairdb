use crate::core::prelude::*;

// Checks if the addition and removal of tags is permitted.
//
// Returns a list with the ids of other organizations that require
// authorization of the pending changes.
//
// If an organization is provided than this organization is excluded
// from both the checks and the pending authorization list.
pub fn authorize_edits<D: Db>(
    db: &D,
    old_tags: &[String],
    new_tags: &[String],
    org: Option<&Organization>,
) -> Result<Vec<Id>> {
    let org_id = org.map(|org| &org.id);
    ofdb_core::authorization::moderated_tag::authorize_edits(
        db.get_moderated_tags_by_org(org_id)?,
        old_tags,
        new_tags,
    )
    .map_err(|_| ParameterError::ModeratedTag.into())
}
