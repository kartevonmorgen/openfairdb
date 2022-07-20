use super::prelude::*;

pub fn archive_ratings<D: Db>(db: &D, user_email: &str, ids: &[&str]) -> Result<usize> {
    log::debug!("Archiving ratings {:?}", ids);
    // TODO: Pass an authentication token with user id and role to
    // check if the user is authorized to perform this use case
    let user = db.try_get_user_by_email(user_email)?;
    if let Some(user) = user {
        if user.role >= Role::Scout {
            let archived = Activity::now(Some(user_email.into()));
            db.archive_comments_of_ratings(ids, &archived)?;
            return Ok(db.archive_ratings(ids, &archived)?);
        }
    }
    Err(Error::Forbidden)
}
