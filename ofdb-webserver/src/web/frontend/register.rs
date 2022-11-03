use maud::Markup;
use rocket::{
    self,
    form::Form,
    get, post,
    request::FlashMessage,
    response::{Flash, Redirect},
    uri, State,
};

use super::{login::LoginCredentials, view};
use crate::{
    core::{prelude::*, usecases},
    web::{guards::*, sqlite::Connections},
};
use ofdb_core::usecases::Error as ParameterError;

#[get("/register")]
pub fn get_register(flash: Option<FlashMessage>) -> Markup {
    view::register(flash)
}

#[allow(clippy::result_large_err)]
#[post("/register", data = "<credentials>")]
pub fn post_register(
    db: Connections,
    notify: &State<Notify>,
    credentials: Form<LoginCredentials>,
) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
    match db.exclusive() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_register)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        //TODO: move into flow layer
        Ok(mut db) => {
            let credentials = credentials.into_inner();
            match usecases::register_with_email(&mut db, &credentials.as_login()) {
                Err(err) => {
                    let msg = match err {
                        ParameterError::UserExists => {
                            "A user with your email address already exists."
                        }
                        ParameterError::Credentials => "Invalid email or password.",
                        _ => "We are so sorry, something went wrong :(",
                    };
                    Err(Flash::error(Redirect::to(uri!(get_register)), msg))
                }
                Ok(()) => {
                    if let Ok(user) = db.get_user_by_email(credentials.email) {
                        debug_assert_eq!(user.email, credentials.email);
                        notify.user_registered_ofdb(&user);

                        let msg = "Registered successfully. Please confirm your email address.";
                        return Ok(Flash::success(
                            Redirect::to(uri!(super::login::get_login)),
                            msg,
                        ));
                    }
                    Err(Flash::error(
                        Redirect::to(uri!(get_register)),
                        "We are so sorry, something went wrong :(",
                    ))
                }
            }
        }
    }
}

#[allow(clippy::result_large_err)]
#[get("/register/confirm/<token>")]
pub fn get_email_confirmation(
    db: Connections,
    token: &str,
) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
    match db.exclusive() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_email_confirmation(token))),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        Ok(db) => match usecases::confirm_email_address(&db, token) {
            Ok(_) => Ok(Flash::success(
                Redirect::to(uri!(super::login::get_login())),
                "Your email address is now confirmed :)",
            )),
            Err(_) => Err(Flash::error(
                Redirect::to(uri!(get_email_confirmation(token))),
                "We are sorry but seems to be something wrong.",
            )),
        },
    }
}
