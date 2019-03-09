use super::view;
use crate::{
    core::{prelude::*, usecases},
    infrastructure::{db::sqlite, error::*, flows::prelude::*},
    ports::web::{guards::*, tantivy::SearchEngine},
};
use maud::Markup;
use rocket::{
    self,
    http::RawStr,
    request::{FlashMessage, Form},
    response::{
        content::{Css, JavaScript},
        Flash, Redirect,
    },
    Route,
};

#[derive(FromForm)]
pub struct RequestNewPw {
    email: String,
}

#[get("/request-new-password")]
pub fn get_request_new_pw(flash: Option<FlashMessage>) -> Markup {
    view::request_new_pw(flash)
}

#[post("/users/actions/request-new-password", data = "<data>")]
pub fn post_request_new_pw(data: Form<RequestNewPw>) -> Flash<Redirect> {
    unimplemented!()
}

#[derive(FromForm)]
pub struct ResetPw {
    nonce: String,
    password: String,
}

#[get("/reset-password?<nonce>")]
pub fn get_reset_pw(flash: Option<FlashMessage>, nonce: &RawStr) -> Markup {
    view::reset_pw(flash, nonce.as_str())
}

#[post("/users/actions/reset-password", data = "<data>")]
pub fn post_reset_pw(data: Form<ResetPw>) -> Flash<Redirect> {
    unimplemented!()
}
