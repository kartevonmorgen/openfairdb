mod dashboard;
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
    Dashboard,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/login",
            Self::Register => "/register",
            Self::ResetPassword => "/reset-password",
            Self::Dashboard => "/dashboard",
        }
    }
}

pub use self::{dashboard::*, home::*, login::*, register::*, reset_password::*};
