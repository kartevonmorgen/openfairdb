use super::api::COOKIE_USER_KEY;
use super::sqlite::DbConn;
use crate::core::error::Error;
use crate::core::usecases;
use maud::Markup;
use maud::PreEscaped;
use rocket::http::{ContentType, Status};
use rocket::http::{Cookie, Cookies};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FlashMessage, Form, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::response::{Responder, Response};
use rocket::Route;
use std::io::Cursor;

mod view;
// --- START WORKAROUND --- //
// As long maud does not support Rocket v0.4.x
// we have to implement `Responder` ourself.

// We need a wrapper because we can't implement traits
// for foreign types.
struct Html(PreEscaped<String>);

impl Responder<'static> for Html {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        Response::build()
            .header(ContentType::HTML)
            .sized_body(Cursor::new((self.0).0))
            .ok()
    }
}
impl From<Markup> for Html {
    fn from(m: Markup) -> Html {
        Html(m)
    }
}
// --- END WORKAROUND --- //

#[derive(FromForm)]
struct Login {
    username: String,
    password: String,
}

#[derive(Debug)]
struct User(String);

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, !> {
        request
            .cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|id| User(id))
            .or_forward(())
    }
}

impl From<Login> for usecases::Login {
    fn from(login: Login) -> usecases::Login {
        let Login { username, password } = login;
        usecases::Login { username, password }
    }
}

#[post("/login", data = "<login>")]
fn login(
    mut db: DbConn,
    mut cookies: Cookies,
    login: Form<Login>,
) -> Result<Redirect, Flash<Redirect>> {
    match usecases::login(&mut *db, &login.into_inner().into()) {
        Ok(username) => {
            cookies.add_private(Cookie::new(COOKIE_USER_KEY, username));
            Ok(Redirect::to(uri!("/admin", index)))
        }
        Err(err) => {
            let msg = match err {
                Error::Parameter(_) => "Invalid username/password.",
                _ => "Internal Server Error.",
            };
            Err(Flash::error(Redirect::to(uri!("/admin", login_page)), msg))
        }
    }
}

#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Flash::success(
        Redirect::to(uri!("/admin", login_page)),
        "Successfully logged out.",
    )
}

#[get("/login")]
fn login_user(_user: User) -> Redirect {
    Redirect::to(uri!("/admin", index))
}

#[get("/login", rank = 2)]
fn login_page(flash: Option<FlashMessage>) -> Html {
    let flash: Option<&str> = match flash {
        Some(ref x) => Some(x.msg()),
        None => None,
    };
    view::index(flash).into()
}

#[get("/")]
fn user_index(user: User) -> Html {
    view::dashboard(&user.0).into()
}

#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to(uri!("/admin", login_page))
}

pub fn routes() -> Vec<Route> {
    routes![login, logout, login_user, login_page, user_index, index,]
}
