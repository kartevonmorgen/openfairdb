use super::{
    login::{AdminLogin as Admin, UserLogin as User},
    sqlite::DbConn,
};
use crate::core::{error::Error, usecases};
use maud::{Markup, PreEscaped};
use rocket::{
    http::{ContentType, Cookies, Status},
    request::{FlashMessage, Form, Request},
    response::{Flash, Redirect},
    response::{Responder, Response},
    Route,
};
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

#[post("/login", data = "<login>")]
fn login(
    db: DbConn,
    mut cookies: Cookies,
    login: Form<usecases::Credentials>,
) -> Result<Redirect, Flash<Redirect>> {
    match super::login::login(&*db, &mut cookies, login.into_inner()) {
        Ok(_) => Ok(Redirect::to(uri!("/admin", index))),
        Err(err) => {
            let msg = match err {
                Error::Parameter(_) => "Invalid username/password.",
                _ => {
                    warn!("{}", err);
                    "Internal Server Error."
                }
            };
            Err(Flash::error(Redirect::to(uri!("/admin", login_page)), msg))
        }
    }
}

#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    super::login::logout(&mut cookies);
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
    let flash: Option<Result<&str, &str>> = match flash {
        Some(ref x) => match x.name() {
            "error" => Some(Err(x.msg())),
            _ => Some(Ok(x.msg())),
        },
        None => None,
    };
    view::index(flash).into()
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to(uri!("/admin", admin_dashboard))
}

#[get("/dashboard")]
fn admin_dashboard(db: DbConn, admin: Admin) -> Html {
    let stats = usecases::get_stats(&*db);
    match stats {
        Ok(stats) => view::admin_dashboard(&admin.0, stats).into(),
        Err(err) => view::admin_dashboard_error(&admin.0, &format!("{}", err)).into(),
    }
}

#[get("/dashboard", rank = 2)]
fn user_dashboard(user: User) -> Html {
    view::user_dashboard(&user.0).into()
}

#[get("/dashboard", rank = 3)]
fn dashboard() -> Redirect {
    Redirect::to(uri!("/admin", login_page))
}

pub fn routes() -> Vec<Route> {
    routes![
        login,
        logout,
        login_user,
        login_page,
        dashboard,
        user_dashboard,
        admin_dashboard,
        index
    ]
}
