use super::login::LoginCredentials;
use super::view;
use crate::{
    core::{prelude::*, usecases},
    infrastructure::notify,
    ports::web::sqlite::Connections,
};
use maud::Markup;
use rocket::{
    self,
    http::RawStr,
    request::{FlashMessage, Form},
    response::{Flash, Redirect},
};

#[get("/register")]
pub fn get_register(flash: Option<FlashMessage>) -> Markup {
    view::register(flash)
}

#[post("/register", data = "<credentials>")]
pub fn post_register(
    db: Connections,
    credentials: Form<LoginCredentials>,
) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
    match db.exclusive() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_register)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        Ok(mut db) => {
            let credentials = credentials.into_inner();
            match usecases::register_with_email(&mut *db, &credentials.as_login()) {
                Err(err) => {
                    let msg = match err {
                        Error::Parameter(ParameterError::UserExists) => {
                            "A user with your email address already exists."
                        }
                        Error::Parameter(ParameterError::Credentials) => {
                            "Invalid email or password."
                        }
                        _ => "We are so sorry, something went wrong :(",
                    };
                    Err(Flash::error(Redirect::to(uri!(get_register)), msg))
                }
                Ok(_) => {
                    if let Ok(user) = db.get_user_by_email(&credentials.email) {
                        debug!("Created user with ID = {}", user.id);

                        debug_assert_eq!(user.email, credentials.email);
                        notify::user_registered_ofdb(&user);
                    }

                    let msg = "Registered sucessfully. Please confirm your email address.";
                    Ok(Flash::success(
                        Redirect::to(uri!(super::login::get_login)),
                        msg,
                    ))
                }
            }
        }
    }
}

#[get("/register/confirm/<user_id>")]
pub fn get_email_confirmation(
    db: Connections,
    user_id: &RawStr,
) -> std::result::Result<Flash<Redirect>, Flash<Redirect>> {
    match db.exclusive() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_email_confirmation: user_id)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        Ok(db) => {
            let u_id = user_id.as_str();
            match usecases::confirm_email_address(&*db, &u_id) {
                Ok(_) => Ok(Flash::success(
                    Redirect::to(uri!(super::login::get_login)),
                    "Your email address is now confirmed :)",
                )),
                Err(_) => Err(Flash::error(
                    Redirect::to(uri!(get_email_confirmation: user_id)),
                    "We are sorry but seems to be something wrong.",
                )),
            }
        }
    }
}
