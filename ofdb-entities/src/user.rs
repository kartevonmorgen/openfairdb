use num_derive::{FromPrimitive, ToPrimitive};

use crate::{email::EmailAddress, password::Password};

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub email           : EmailAddress,
    pub email_confirmed : bool,
    pub password        : Password,
    pub role            : Role,
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum Role {
    #[default]
    Guest = 0,
    User  = 1,
    Scout = 2,
    Admin = 3,
}
