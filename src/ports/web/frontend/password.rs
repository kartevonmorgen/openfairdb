use maud::Markup;
use rocket::{
    self,
    form::Form,
    get, post,
    request::FlashMessage,
    response::{Flash, Redirect},
    uri, FromForm,
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
    token: Option<&str>,
    success: Option<&str>,
) -> Markup {
    let success = success.map(|s| s == "true" || s == "1");

    if let Some(token) = token {
        if let Some(true) = success {
            view::reset_password_ack(flash)
        } else {
            view::reset_password(flash, "/users/actions/reset-password", token)
        }
    } else if let Some(true) = success {
        view::reset_password_request_ack(flash)
    } else {
        view::reset_password_request(flash, "/users/actions/reset-password-request")
    }
}

#[derive(FromForm)]
pub struct ResetPasswordRequest<'r> {
    email: &'r str,
}

#[post("/users/actions/reset-password-request", data = "<data>")]
pub fn post_reset_password_request(
    db: Connections,
    notify: Notify,
    data: Form<ResetPasswordRequest>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let ResetPasswordRequest { email } = data.into_inner();
    match reset_password_request(&db, &*notify, email) {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_reset_password(_, _))),
            "Failed to request a password reset.",
        )),
        Ok(_) => Ok(Redirect::to(uri!(get_reset_password(
            token = _,
            success = Some("true")
        )))),
    }
}

#[derive(FromForm)]
pub struct ResetPassword<'r> {
    token: &'r str,
    new_password: &'r str,
    new_password_repeated: &'r str,
}

#[post("/users/actions/reset-password", data = "<data>")]
pub fn post_reset_password(
    db: Connections,
    data: Form<ResetPassword>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let req = data.into_inner();

    if req.new_password != req.new_password_repeated {
        return Err(Flash::error(
            Redirect::to(uri!(get_reset_password(
                token = Some(req.token),
                success = _
            ))),
            "Your passwords do not match.",
        ));
    }
    match req.new_password.parse::<Password>() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_reset_password(
                token = Some(req.token),
                success = _
            ))),
            "Your new password is not allowed.",
        )),
        Ok(new_password) => match EmailNonce::decode_from_str(req.token) {
            Err(_) => Err(Flash::error(
                Redirect::to(uri!(get_reset_password(
                    token = Some(req.token),
                    success = _
                ))),
                "Resetting your password is not possible (invalid token).",
            )),
            Ok(email_nonce) => {
                match reset_password_with_email_nonce(&db, email_nonce, new_password) {
                    Err(_) => Err(Flash::error(
                        Redirect::to(uri!(get_reset_password(
                            token = Some(req.token),
                            success = _
                        ))),
                        "Failed to request a password reset.",
                    )),
                    Ok(_) => Ok(Redirect::to(uri!(get_reset_password(
                        token = Some(req.token),
                        success = Some("true")
                    )))),
                }
            }
        },
    }
}
