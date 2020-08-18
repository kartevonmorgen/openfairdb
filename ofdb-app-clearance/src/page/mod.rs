use crate::Msg;
use seed::{prelude::*, *};

pub mod index;
pub mod login;

#[derive(Debug)]
pub enum Page {
    Home(index::Mdl),
    Login(login::Mdl),
    NotFound,
}

impl Page {
    pub fn init(mut url: Url, orders: &mut impl Orders<Msg>) -> Self {
        match url.next_hash_path_part() {
            None => index::init(url, &mut orders.proxy(Msg::PageIndex))
                .map_or(Self::NotFound, Self::Home),
            Some("login") => login::init(url).map_or(Self::NotFound, Self::Login),
            _ => {
                log!("not found:", url);
                Self::NotFound
            }
        }
    }
}
