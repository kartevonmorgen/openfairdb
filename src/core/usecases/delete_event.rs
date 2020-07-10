use crate::core::prelude::*;

pub fn delete_event<D: Db>(db: &mut D, token: &str, id: &str) -> Result<()> {
    let org = db.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    let moderated_tags: Vec<_> = org
        .moderated_tags
        .iter()
        .map(|moderated_tag| moderated_tag.label.as_str())
        .collect();
    // FIXME: Only events with at least one tag that is owned by
    // the organization can be deleted. If the organization
    // doesn't own any tags deletion of events must not be
    // permitted!
    /*
    if moderated_tags.is_empty() {
        return Err(Error::Parameter(ParameterError::ModeratedTag));
    }
    */
    db.delete_event_with_matching_tags(id, &moderated_tags)?
        .ok_or(Error::Parameter(ParameterError::ModeratedTag))
}
