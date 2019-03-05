use crate::core::prelude::*;
use pwhash::bcrypt;

//TODO: remove and use Credentials instead
#[derive(Deserialize, Debug, Clone)]
pub struct Login {
    username: String,
    password: String,
}

pub struct Credentials<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

//TODO: remove and use email instead
pub fn login_with_username<D: Db>(db: &D, login: &Login) -> Result<String> {
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

pub fn login_with_email<D: Db>(db: &D, login: &Credentials) -> Result<Role> {
    match db.get_user_by_email(&login.email) {
        Ok(u) => {
            if bcrypt::verify(&login.password, &u.password) {
                if u.email_confirmed {
                    Ok(u.role)
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
