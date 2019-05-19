use crate::core::prelude::*;

pub fn archive_comments<D: Db>(db: &D, user_email: &str, ids: &[&str]) -> Result<()> {
    info!("Archiving {} comments", ids.len());
    let users = db.get_users_by_email(user_email)?;
    if let Some(user) = users.first() {
        if user.role >= Role::Scout {
            let archived = Timestamp::now();
            db.archive_comments(ids, archived)?;
            return Ok(());
        }
    }
    Err(ParameterError::Forbidden.into())
}
