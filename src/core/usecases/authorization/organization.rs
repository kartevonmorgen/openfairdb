use crate::core::prelude::*;

pub fn authorize_by_token<D: Db>(db: &D, token: &str) -> Result<Organization> {
    db.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })
}
