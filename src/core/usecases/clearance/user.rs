use crate::core::prelude::*;

pub fn authorize_by_email(db: &dyn Db, email: &str, min_required_role: Role) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(email)? {
        return ofdb_core::clearance::user::authorize_role(&user, min_required_role)
            .map(|()| user)
            .map_err(|_| Error::Parameter(ParameterError::Unauthorized));
    }
    Err(Error::Parameter(ParameterError::Unauthorized))
}
