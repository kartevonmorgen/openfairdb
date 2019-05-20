use crate::core::prelude::*;

pub fn change_user_role<D: Db>(
    db: &D,
    account_email: &str,
    user_email: &str,
    role: Role,
) -> Result<()> {
    info!("Changing role to {:?} for {}", role, user_email);
    // TODO: Pass an authentication token with user id and role
    // instead of account_email to check if this user is authorized
    // to perform this use case.
    let accounts = db.get_users_by_email(account_email)?;
    let users = db.get_users_by_email(user_email)?;
    let account = accounts
        .first()
        .ok_or_else(|| ParameterError::UserDoesNotExist)?;
    let mut user = users
        .first()
        .ok_or_else(|| ParameterError::UserDoesNotExist)?
        .to_owned();
    if account.role > user.role && role < account.role {
        user.role = role;
        db.update_user(&user)?;
        Ok(())
    } else {
        Err(ParameterError::Forbidden.into())
    }
}
