mod dashboard;
mod entry;
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
    Entries,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/login",
            Self::Register => "/register",
            Self::ResetPassword => "/reset-password",
            Self::Dashboard => "/dashboard",
            Self::Entries => "/entries",
        }
    }
}

pub use self::{dashboard::*, entry::*, home::*, login::*, register::*, reset_password::*};
