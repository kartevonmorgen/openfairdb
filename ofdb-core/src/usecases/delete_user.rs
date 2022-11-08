use super::prelude::*;

pub fn delete_user<R>(repo: &R, login_email: &EmailAddress, email: &EmailAddress) -> Result<()>
where
    R: UserRepo,
{
    if login_email != email {
        return Err(Error::Forbidden);
    }
    Ok(repo.delete_user_by_email(email)?)
}
