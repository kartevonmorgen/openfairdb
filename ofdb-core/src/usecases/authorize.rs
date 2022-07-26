use super::prelude::*;
use crate::repositories::Error as RepoError;

pub fn authorize_organization_by_possible_api_tokens<D: OrganizationRepo>(
    db: &D,
    tokens: &[String],
) -> Result<Organization> {
    for token in tokens {
        match db.get_org_by_api_token(token) {
            Ok(org) => return Ok(org),
            Err(RepoError::NotFound) => (),
            Err(e) => return Err(Error::Repo(e)),
        }
    }
    Err(Error::Unauthorized)
}

pub fn authorize_user_by_email(db: &dyn Db, email: &str, min_required_role: Role) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(email)? {
        return crate::user::authorize_role(&user, min_required_role)
            .map(|()| user)
            .map_err(|_| Error::Unauthorized);
    }
    Err(Error::Unauthorized)
}

// Checks if the addition and removal of tags is permitted.
//
// Returns a list with the ids of other organizations that require
// clearance of the pending changes.
//
// If an organization is provided than this organization is excluded
// from both the checks and the pending clearance list.
pub fn authorize_editing_of_tagged_entry<R: OrganizationRepo>(
    repo: &R,
    old_tags: &[String],
    new_tags: &[String],
    org: Option<&Organization>,
) -> Result<Vec<Id>> {
    let org_id = org.map(|org| &org.id);
    let moderated_tags_by_org = repo.get_moderated_tags_by_org(org_id)?;
    crate::tag::moderated::authorize_editing_of_tagged_entry(
        moderated_tags_by_org,
        old_tags,
        new_tags,
    )
    .map_err(|_| Error::ModeratedTag)
}
