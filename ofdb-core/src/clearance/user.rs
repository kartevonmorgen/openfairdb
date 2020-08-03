use ofdb_entities::user::{Role, User};

use std::result::Result as StdResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unauthorized role")]
    UnauthorizedRole,
}

pub type Result<T> = StdResult<T, Error>;

pub fn authorize_role(user: &User, min_required_role: Role) -> Result<()> {
    if user.role < min_required_role {
        return Err(Error::UnauthorizedRole);
    }
    Ok(())
}
