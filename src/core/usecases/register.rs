use super::Credentials;
use crate::core::prelude::*;

pub fn register_with_email<D: UserGateway>(db: &mut D, credentials: &Credentials) -> Result<()> {
    let password = credentials.password.to_string();
    let email = credentials.email.to_string();
    let new_user = super::NewUser { email, password };
    super::create_new_user(db, new_user)?;
    Ok(())
}
