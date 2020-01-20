use crate::password::Password;
use num_derive::{FromPrimitive, ToPrimitive};

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub email           : String,
    pub email_confirmed : bool,
    pub password        : Password,
    pub role            : Role,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum Role {
    Guest = 0,
    User  = 1,
    Scout = 2,
    Admin = 3,
}

impl Default for Role {
    fn default() -> Role {
        Role::Guest
    }
}
