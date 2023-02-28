mod home;
mod login;
mod register;
mod reset_password;

#[derive(Debug, Clone, Copy, Default)]
pub enum Page {
    #[default]
    Home,
    Login,
    Register,
    ResetPassword,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/login",
            Self::Register => "/register",
            Self::ResetPassword => "/reset-password",
        }
    }
}

pub use self::{home::*, login::*, register::*, reset_password::*};
