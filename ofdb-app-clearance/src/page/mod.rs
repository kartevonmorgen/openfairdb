use seed::{prelude::*, *};

use crate::Msg;

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
            None => match index::init(&mut orders.proxy(Msg::PageIndex)) {
                Some(mdl) => Self::Home(mdl),
                None => {
                    let url = Url::new()
                        .set_path(&[crate::PAGE_URL])
                        .set_hash_path(&[crate::HASH_PATH_LOGIN]);
                    orders.request_url(url);
                    Self::NotFound
                }
            },
            Some(crate::HASH_PATH_LOGIN) => Self::Login(login::init(url)),
            _ => {
                log!("not found:", url);
                Self::NotFound
            }
        }
    }
}
