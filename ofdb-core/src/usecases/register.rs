use crate::usecases::{
    create_new_user::{create_new_user, NewUser},
    login::Credentials,
    prelude::*,
};

pub fn register_with_email<D: UserRepo>(db: &mut D, credentials: &Credentials) -> Result<()> {
    let password = credentials.password.to_string();
    let email = credentials.email.to_owned();
    let new_user = NewUser { email, password };
    create_new_user(db, new_user)?;
    Ok(())
}
