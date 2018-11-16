use crate::core::prelude::*;
use pwhash::bcrypt;

#[derive(Deserialize, Debug, Clone)]
pub struct Login {
    username: String,
    password: String,
}

pub fn login<D: Db>(db: &mut D, login: &Login) -> Result<String> {
    match db.get_user(&login.username) {
        Ok(u) => {
            if bcrypt::verify(&login.password, &u.password) {
                if u.email_confirmed {
                    Ok(login.username.clone())
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
