use crate::core::prelude::*;

pub fn archive_ratings<D: Db>(db: &D, user_email: &str, ids: &[&str]) -> Result<()> {
    debug!("Archiving ratings {:?}", ids);
    // TODO: Pass an authentication token with user id and role to
    // check if the user is authorized to perform this use case
    let user = db.try_get_user_by_email(user_email)?;
    if let Some(user) = user {
        if user.role >= Role::Scout {
            let archived = Timestamp::now();
            db.archive_comments_of_ratings(ids, archived)?;
            db.archive_ratings(ids, archived)?;
            return Ok(());
        }
    }
    Err(ParameterError::Forbidden.into())
}
