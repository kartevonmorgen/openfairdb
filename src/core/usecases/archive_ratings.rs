use crate::core::prelude::*;

pub fn archive_ratings<D: Db>(db: &D, user_email: &str, ids: &[&str]) -> Result<()> {
    debug!("Archiving ratings {:?}", ids);
    let users = db.get_users_by_email(user_email)?;
    if let Some(user) = users.first() {
        if user.role >= Role::Scout {
            let archived = Timestamp::now();
            db.archive_comments_of_ratings(ids, archived)?;
            db.archive_ratings(ids, archived)?;
            return Ok(());
        }
    }
    Err(ParameterError::Forbidden.into())
}
