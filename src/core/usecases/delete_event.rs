use crate::core::prelude::*;

pub fn delete_event<D: Db>(db: &mut D, id: &str, token: &str) -> Result<()> {
    let _org = db.get_org_by_api_token(token).map_err(|e| match e {
        RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
        _ => Error::Repo(e),
    })?;
    db.delete_event(id)?;
    Ok(())
}
