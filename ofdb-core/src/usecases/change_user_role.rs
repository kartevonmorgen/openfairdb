use super::prelude::*;

pub fn change_user_role<R>(
    repo: &R,
    account_email: &EmailAddress,
    user_email: &EmailAddress,
    role: Role,
) -> Result<()>
where
    R: UserRepo,
{
    log::info!("Changing role to {:?} for {}", role, user_email);
    // TODO: Pass an authentication token with user id and role
    // instead of account_email to check if this user is authorized
    // to perform this use case.
    let account = repo
        .try_get_user_by_email(account_email)?
        .ok_or(Error::UserDoesNotExist)?;
    let mut user = repo
        .try_get_user_by_email(user_email)?
        .ok_or(Error::UserDoesNotExist)?;
    if account.role > user.role && role < account.role {
        user.role = role;
        repo.update_user(&user)?;
        Ok(())
    } else {
        Err(Error::Forbidden)
    }
}
