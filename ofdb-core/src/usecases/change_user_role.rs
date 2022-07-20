use super::prelude::*;

pub fn change_user_role<D: Db>(
    db: &D,
    account_email: &str,
    user_email: &str,
    role: Role,
) -> Result<()> {
    log::info!("Changing role to {:?} for {}", role, user_email);
    // TODO: Pass an authentication token with user id and role
    // instead of account_email to check if this user is authorized
    // to perform this use case.
    let account = db
        .try_get_user_by_email(account_email)?
        .ok_or(Error::UserDoesNotExist)?;
    let mut user = db
        .try_get_user_by_email(user_email)?
        .ok_or(Error::UserDoesNotExist)?;
    if account.role > user.role && role < account.role {
        user.role = role;
        db.update_user(&user)?;
        Ok(())
    } else {
        Err(Error::Forbidden)
    }
}
