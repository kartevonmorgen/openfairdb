use super::prelude::*;

pub fn confirm_email_and_reset_password<R>(
    repo: &R,
    email: &str,
    new_password: Password,
) -> Result<()>
where
    R: UserRepo,
{
    log::info!("Resetting password for user ({})", email);
    let mut user = repo.get_user_by_email(email)?;
    debug_assert_eq!(user.email, email);
    user.email_confirmed = true;
    user.password = new_password;
    repo.update_user(&user)?;
    Ok(())
}
