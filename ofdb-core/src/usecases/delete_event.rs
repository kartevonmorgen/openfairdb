use super::prelude::*;
use crate::repositories::Error as RepoError;

pub fn delete_event<R>(repo: &R, token: &str, id: &str) -> Result<()>
where
    R: OrganizationRepo + EventRepo,
{
    let org = repo.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Unauthorized,
        _ => Error::Repo(e),
    })?;
    let moderated_tags: Vec<_> = org
        .moderated_tags
        .iter()
        .map(|moderated_tag| moderated_tag.label.as_str())
        .collect();
    if moderated_tags.is_empty() && repo.is_event_owned_by_any_organization(id)? {
        // Prevent deletion of events owned by another organization
        // if the given organization does not own any tags.
        return Err(Error::ModeratedTag);
    }
    let deleted = repo.delete_event_with_matching_tags(id, &moderated_tags)?;
    if !deleted {
        // No matching tags, i.e. event is not owned by the given organization
        return Err(Error::ModeratedTag);
    }
    Ok(())
}
