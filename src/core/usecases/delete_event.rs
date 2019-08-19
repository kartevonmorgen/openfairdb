use crate::core::prelude::*;

pub fn delete_event<D: Db>(db: &mut D, id: &str, token: &str) -> Result<()> {
    let org = db.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    let tags: Vec<_> = org.owned_tags.iter().map(|tag| tag.as_str()).collect();
    db.delete_event_with_matching_tags(id, &tags)
        .map_err(|e| match e {
            RepoError::NotFound => {
                if db.get_event(id).is_ok() {
                    // Event actually exists, so tags didn't match
                    Error::Parameter(ParameterError::Unauthorized)
                } else {
                    // Really not found
                    Error::Repo(RepoError::NotFound)
                }
            }
            e => Error::Repo(e),
        })
}
