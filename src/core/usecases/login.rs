use crate::core::prelude::*;

//TODO: remove and use Credentials instead
#[derive(Deserialize, Debug, Clone)]
pub struct Login {
    pub email: String,
    pub password: String,
}

pub struct Credentials<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

pub fn login_with_email<D: Db>(db: &D, login: &Credentials) -> Result<Role> {
    db.try_get_user_by_email(&login.email)
        .map_err(Error::Repo)
        .and_then(|user| {
            if let Some(u) = user {
                if u.password.verify(&login.password) {
                    if u.email_confirmed {
                        Ok(u.role)
                    } else {
                        Err(Error::Parameter(ParameterError::EmailNotConfirmed))
                    }
                } else {
                    Err(Error::Parameter(ParameterError::Credentials))
                }
            } else {
                Err(Error::Parameter(ParameterError::Credentials))
            }
        })
}

#[cfg(test)]
mod tests {
    //TODO: write tests
}
