use maud::Markup;
use rocket::{
    self,
    http::RawStr,
    request::{FlashMessage, Form},
    response::{Flash, Redirect},
};

use super::view;
use crate::{
    core::prelude::*,
    infrastructure::flows::prelude::*,
    ports::web::{notify::*, sqlite::Connections},
};

#[get("/reset-password?<token>&<success>")]
pub fn get_reset_password(
    flash: Option<FlashMessage>,
    token: Option<&RawStr>,
    success: Option<&RawStr>,
) -> Markup {
    let success = success
        .map(|raw| raw.as_str())
        .map(|s| s == "true" || s == "1");

    if let Some(token) = token {
        if let Some(true) = success {
            view::reset_password_ack(flash)
        } else {
            view::reset_password(flash, "/users/actions/reset-password", token.as_str())
        }
    } else if let Some(true) = success {
        view::reset_password_request_ack(flash)
    } else {
        view::reset_password_request(flash, "/users/actions/reset-password-request")
    }
}

#[derive(FromForm)]
pub struct ResetPasswordRequest {
    email: String,
}

#[post("/users/actions/reset-password-request", data = "<data>")]
pub fn post_reset_password_request(
    db: Connections,
    notify: Notify,
    data: Form<ResetPasswordRequest>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let ResetPasswordRequest { email } = data.into_inner();
    match reset_password_request(&db, &*notify, &email) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_reset_password: token = _, success = _)),
            "Failed to request a password reset.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(
            get_reset_password: token = _,
            success = "true"
        ))),
    }
}

#[derive(FromForm)]
pub struct ResetPassword {
    token: String,
    new_password: String,
    new_password_repeated: String,
}

#[post("/users/actions/reset-password", data = "<data>")]
pub fn post_reset_password(
    db: Connections,
    data: Form<ResetPassword>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let req = data.into_inner();

    if req.new_password != req.new_password_repeated {
        return Err(Flash::error(
            Redirect::to(uri!(get_reset_password: token = req.token, success = _)),
            "Your passwords do not match.",
        ));
    }
    match req.new_password.parse::<Password>() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_reset_password: token = req.token, success = _)),
            "Your new password is not allowed.",
        )),
        Ok(new_password) => match EmailNonce::decode_from_str(&req.token) {
            Err(_) => Err(Flash::error(
                Redirect::to(uri!(get_reset_password: token = req.token, success = _)),
                "Resetting your password is not possible (invalid token).",
            )),
            Ok(email_nonce) => {
                match reset_password_with_email_nonce(&db, email_nonce, new_password) {
                    Err(_) => Err(Flash::error(
                        Redirect::to(uri!(get_reset_password: token = req.token, success = _)),
                        "Failed to request a password reset.",
                    )),
                    Ok(_) => Ok(Redirect::to(uri!(
                        get_reset_password: token = req.token,
                        success = "true"
                    ))),
                }
            }
        },
    }
}
