use super::*;

pub fn confirm_email_and_reset_password<D: Db>(
    db: &D,
    email: &str,
    new_password: Password,
) -> Result<()> {
    info!("Resetting password for user ({})", email);
    let mut user = db.get_user_by_email(email)?;
    debug_assert_eq!(user.email, email);
    user.email_confirmed = true;
    user.password = new_password;
    db.update_user(&user)?;
    Ok(())
}
