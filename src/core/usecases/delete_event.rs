use crate::core::prelude::*;

pub fn delete_event<D: Db>(db: &mut D, token: &str, id: &str) -> Result<()> {
    let org = db.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    let owned_tags: Vec<_> = org.owned_tags.iter().map(|tag| tag.as_str()).collect();
    // FIXME: Only events with at least one tag that is owned by
    // the organization can be deleted. If the organization
    // doesn't own any tags deletion of events must not be
    // permitted!
    /*
    if owned_tags.is_empty() {
        return Err(Error::Parameter(ParameterError::OwnedTag));
    }
    */
    db.delete_event_with_matching_tags(id, &owned_tags)?
        .ok_or(Error::Parameter(ParameterError::OwnedTag))
}
