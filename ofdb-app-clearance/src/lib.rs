use seed::{prelude::*, *};

mod api;
mod page;

use page::Page;

const TOKEN_KEY: &str = "org-token";
const TITLE: &str = "Clearance Center";
const PAGE_URL: &str = "clearance";
const HASH_PATH_LOGIN: &str = "login";
const HASH_PATH_INVALID: &str = "invalid";

#[derive(Debug)]
struct Mdl {
    page: Page,
}

#[derive(Clone)]
pub enum Msg {
    UrlChanged(subs::UrlChanged),
    PageIndex(page::index::Msg),
    PageLogin(page::login::Msg),
}

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            mdl.page = Page::init(url, orders);
        }
        Msg::PageIndex(msg) => {
            if let Page::Home(home_mdl) = &mut mdl.page {
                page::index::update(msg, home_mdl, &mut orders.proxy(Msg::PageIndex));
            }
        }
        Msg::PageLogin(msg) => {
            if let Page::Login(login_mdl) = &mut mdl.page {
                page::login::update(msg, login_mdl, &mut orders.proxy(Msg::PageLogin));
            }
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Mdl {
    orders.subscribe(Msg::UrlChanged);
    Mdl {
        page: Page::init(url, orders),
    }
}

fn view(mdl: &Mdl) -> Node<Msg> {
    match &mdl.page {
        Page::Home(mdl) => page::index::view(&mdl).map_msg(Msg::PageIndex),
        Page::Login(mdl) => page::login::view(&mdl).map_msg(Msg::PageLogin),
        Page::NotFound => div!["Not Found!"],
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
