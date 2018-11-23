use crate::core::prelude::*;
use pwhash::bcrypt;

#[derive(Deserialize, FromForm)]
pub struct Credentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

pub fn login<D: Db>(db: &D, login: &Credentials) -> Result<(String, AccessLevel)> {
    match db.get_user(&login.username) {
        Ok(u) => {
            if bcrypt::verify(&login.password, &u.password) {
                if u.email_confirmed {
                    Ok((login.username.clone(), u.access))
                } else {
                    Err(Error::Parameter(ParameterError::EmailNotConfirmed))
                }
            } else {
                Err(Error::Parameter(ParameterError::Credentials))
            }
        }
        Err(err) => match err {
            RepoError::NotFound => Err(Error::Parameter(ParameterError::Credentials)),
            _ => Err(Error::Repo(RepoError::Other(Box::new(err)))),
        },
    }
}

#[cfg(test)]
mod tests {
    //TODO: write tests
}
