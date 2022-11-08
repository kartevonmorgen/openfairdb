use super::prelude::*;

pub fn get_user<R>(
    repo: &R,
    logged_in_email: &EmailAddress,
    requested_email: &EmailAddress,
) -> Result<User>
where
    R: UserRepo,
{
    if logged_in_email != requested_email {
        return Err(Error::Forbidden);
    }
    Ok(repo.get_user_by_email(requested_email)?)
}
