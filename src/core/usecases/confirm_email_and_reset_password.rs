use super::*;

use crate::core::error::{Error, ParameterError};

pub fn confirm_email_and_reset_password<D: Db>(
    db: &D,
    username: &str,
    email: &str,
    new_password: Password,
) -> Result<()> {
    info!("Resetting password for user ({})", username);
    let mut user = db.get_user(username)?;
    debug_assert_eq!(user.username, username);
    if user.email != email {
        warn!(
            "Invalid e-mail address for user ({}): expected = {}, actual = {}",
            user.username, user.email, email,
        );
        return Err(Error::Parameter(ParameterError::Email));
    }
    user.email_confirmed = true;
    user.password = new_password;
    db.update_user(&user)?;
    Ok(())
}
