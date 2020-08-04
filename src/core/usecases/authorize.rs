use crate::core::prelude::*;

pub fn authorize_organization_by_api_token<D: Db>(db: &D, api_token: &str) -> Result<Organization> {
    db.get_org_by_api_token(api_token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })
}

pub fn authorize_user_by_email(db: &dyn Db, email: &str, min_required_role: Role) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(email)? {
        return ofdb_core::user::authorize_role(&user, min_required_role)
            .map(|()| user)
            .map_err(|_| Error::Parameter(ParameterError::Unauthorized));
    }
    Err(Error::Parameter(ParameterError::Unauthorized))
}
