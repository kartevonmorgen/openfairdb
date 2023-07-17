pub mod index;
pub mod login;

#[derive(Debug)]
pub enum Page {
    Home,
    Login,
    Logout,
}

impl Page {
    pub fn path(&self) -> &str {
        match self {
            Self::Home => "/clearance",
            Self::Login => "/clearance/login",
            Self::Logout => "/clearance/logout",
        }
    }
}
