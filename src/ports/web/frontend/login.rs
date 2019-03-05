use super::view;
use crate::{
    core::{prelude::*, usecases},
    ports::web::sqlite::Connections,
};
use maud::Markup;
use rocket::{
    self,
    http::{Cookie, Cookies},
    outcome::IntoOutcome,
    request::{self, FlashMessage, Form, FromRequest, Request},
    response::{Flash, Redirect},
};

#[derive(FromForm)]
pub struct LoginCredentials {
    pub email: String,
    password: String,
}

impl<'a> LoginCredentials {
    pub fn as_login(&'a self) -> usecases::Credentials<'a> {
        let LoginCredentials {
            ref email,
            ref password,
        } = self;
        usecases::Credentials { email, password }
    }
}

#[derive(Debug)]
pub struct Account(String);

impl Account {
    pub fn email(&self) -> &str {
        &self.0
    }
}

const COOKIE_EMAIL_KEY: &str = "ofdb-user-email";

impl<'a, 'r> FromRequest<'a, 'r> for Account {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Account, !> {
        request
            .cookies()
            .get_private(COOKIE_EMAIL_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(|email| Account(email))
            .or_forward(())
    }
}

#[get("/login")]
pub fn get_login_user(_account: Account) -> Redirect {
    Redirect::to(uri!(super::get_index))
}

#[get("/login", rank = 2)]
pub fn get_login(flash: Option<FlashMessage>) -> Markup {
    view::login(flash)
}

#[post("/login", data = "<credentials>")]
pub fn post_login(
    db: Connections,
    credentials: Form<LoginCredentials>,
    mut cookies: Cookies,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    match db.shared() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_login)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        Ok(db) => {
            let credentials = credentials.into_inner();
            match usecases::login_with_email(&*db, &credentials.as_login()) {
                Err(err) => {
                    let msg = match err {
                        Error::Parameter(ParameterError::EmailNotConfirmed) => {
                            "You have to confirm your email address first."
                        }
                        Error::Parameter(ParameterError::Credentials) => {
                            "Invalid email or password."
                        }
                        _ => panic!(),
                    };
                    Err(Flash::error(Redirect::to(uri!(get_login)), msg))
                }
                Ok(_) => {
                    cookies.add_private(Cookie::new(COOKIE_EMAIL_KEY, credentials.email));
                    Ok(Redirect::to(uri!(super::get_index)))
                }
            }
        }
    }
}

#[post("/logout")]
pub fn post_logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
    Flash::success(
        Redirect::to(uri!(super::get_index)),
        "Sie haben sich erfolgreich abgemeldet.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::web::tests::prelude::*;

    fn setup() -> (Client, Connections) {
        let (client, db, _) = crate::ports::web::tests::setup(vec![("/", super::super::routes())]);
        (client, db)
    }

    fn user_id_cookie(response: &Response) -> Option<Cookie<'static>> {
        let cookie = response
            .headers()
            .get("Set-Cookie")
            .filter(|v| v.starts_with(COOKIE_EMAIL_KEY))
            .nth(0)
            .and_then(|val| Cookie::parse_encoded(val).ok());
        cookie.map(|c| c.into_owned())
    }

    fn register_user(pool: Connections, email: &str, pw: &str, confirmed: bool) {
        let mut db = pool.exclusive().unwrap();
        usecases::create_new_user(
            &mut *db,
            usecases::NewUser {
                username: email.replace("@", "").replace(".", "").into(),
                email: email.into(),
                password: pw.into(),
            },
        )
        .unwrap();
        if confirmed {
            let u = db.get_user_by_email(email).unwrap();
            usecases::confirm_email_address(&mut *db, &u.id).unwrap();
        }
    }

    #[test]
    fn get_login() {
        let (client, _) = setup();
        let mut res = client.get("/login").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("action=\"login\""));
        assert!(user_id_cookie(&res).is_none());
    }

    #[test]
    fn post_login_fails() {
        let (client, pool) = setup();
        register_user(pool, "foo@bar.com", "baz", true);
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(format!("email=foo%40bar.com&password=invalid"))
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        for h in res.headers().iter() {
            match h.name.as_str() {
                "Location" => assert_eq!(h.value, "/login"),
                "Content-Length" => assert_eq!(h.value.parse::<i32>().unwrap(), 0),
                _ => { /* let these through */ }
            }
        }
    }

    #[test]
    fn post_login_sucess() {
        let (client, pool) = setup();
        register_user(pool, "foo@bar.com", "baz", true);
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body(format!("email=foo%40bar.com&password=baz"))
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        assert!(user_id_cookie(&res).is_some());
        //TODO: extract private cookie value to assert v == "foo@bar.com"
        for h in res.headers().iter() {
            match h.name.as_str() {
                "Location" => assert_eq!(h.value, "/"),
                "Content-Length" => assert_eq!(h.value.parse::<i32>().unwrap(), 0),
                _ => { /* let these through */ }
            }
        }
    }
}
