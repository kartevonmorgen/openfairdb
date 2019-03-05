use super::Credentials;
use crate::core::prelude::*;

pub fn register_with_email<D: UserGateway>(db: &mut D, credentials: &Credentials) -> Result<()> {
    match db.get_user_by_email(credentials.email) {
        Ok(_) => Err(Error::Parameter(ParameterError::UserExists)),
        Err(e) => match e {
            RepoError::NotFound => {
                let username = super::generate_username_from_email(&credentials.email);
                let password = credentials.password.to_string();
                let email = credentials.email.to_string();
                let new_user = super::NewUser {
                    username,
                    password,
                    email,
                };
                super::create_new_user(db, new_user)?;
                Ok(())
            }
            _ => Err(e.into()),
        },
    }
}
