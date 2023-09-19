mod dashboard;
mod entry;
mod event;
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
    Events,
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
            Self::Events => "/events",
        }
    }
}

pub use self::{
    dashboard::*, entry::*, event::*, home::*, login::*, register::*, reset_password::*,
};
