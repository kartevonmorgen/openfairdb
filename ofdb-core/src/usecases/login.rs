use super::prelude::*;

pub struct Credentials<'a> {
    pub email: &'a EmailAddress,
    pub password: &'a str,
}

pub fn login_with_email<R>(repo: &R, login: &Credentials) -> Result<Role>
where
    R: UserRepo,
{
    repo.try_get_user_by_email(login.email)
        .map_err(Error::Repo)
        .and_then(|user| {
            if let Some(u) = user {
                if u.password.verify(login.password) {
                    if u.email_confirmed {
                        Ok(u.role)
                    } else {
                        Err(Error::EmailNotConfirmed)
                    }
                } else {
                    Err(Error::Credentials)
                }
            } else {
                Err(Error::Credentials)
            }
        })
}
