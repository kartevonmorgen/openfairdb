use seed::prelude::*;

mod update;
mod view;

#[derive(Debug, Default)]
pub struct Mdl {
    // TODO
}

#[derive(Clone)]
pub enum Msg {
    // TODO
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Mdl {
    Mdl::default()
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update::update, view::view);
}
