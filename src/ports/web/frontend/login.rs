use maud::Markup;
use rocket::{
    self,
    form::Form,
    get,
    http::{Cookie, CookieJar, SameSite},
    post,
    request::FlashMessage,
    response::{Flash, Redirect},
    uri, FromForm,
};

use super::{super::guards::*, view};
use crate::{
    core::{prelude::*, usecases},
    ports::web::sqlite::Connections,
};

#[derive(FromForm)]
pub struct LoginCredentials<'r> {
    pub email: &'r str,
    password: &'r str,
}

impl LoginCredentials<'_> {
    pub fn as_login(&self) -> usecases::Credentials<'_> {
        let LoginCredentials { email, password } = self;
        usecases::Credentials { email, password }
    }
}

#[get("/login")]
pub fn get_login(
    account: Option<Account>,
    flash: Option<FlashMessage>,
) -> std::result::Result<Markup, Redirect> {
    if account.is_some() {
        Err(Redirect::to(uri!(super::get_index)))
    } else {
        Ok(view::login(flash, "/reset-password"))
    }
}

#[post("/login", data = "<credentials>")]
pub fn post_login(
    db: Connections,
    credentials: Form<LoginCredentials>,
    cookies: &CookieJar<'_>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    match db.shared() {
        Err(_) => Err(Flash::error(
            Redirect::to(uri!(get_login)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        )),
        Ok(db) => match usecases::login_with_email(&*db, &credentials.as_login()) {
            Err(err) => {
                let msg = match err {
                    Error::Parameter(ParameterError::EmailNotConfirmed) => {
                        "You have to confirm your email address first."
                    }
                    Error::Parameter(ParameterError::Credentials) => "Invalid email or password.",
                    _ => panic!(),
                };
                Err(Flash::error(Redirect::to(uri!(get_login)), msg))
            }
            Ok(_) => {
                let email = credentials.email.to_string();
                cookies.add_private(
                    Cookie::build(COOKIE_EMAIL_KEY, email)
                        .http_only(true)
                        .same_site(SameSite::Lax)
                        .finish(),
                );
                Ok(Redirect::to(uri!(super::get_index)))
            }
        },
    }
}

#[post("/logout")]
pub fn post_logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
    Flash::success(
        Redirect::to(uri!(super::get_index)),
        "Sie haben sich erfolgreich abgemeldet.",
    )
}

#[cfg(test)]
pub mod tests {
    use rocket::http::Status as HttpStatus;

    use super::*;
    use crate::ports::web::{
        self,
        tests::{prelude::*, register_user},
    };

    fn setup() -> (Client, Connections) {
        let (client, db, _) = web::tests::setup(vec![("/", super::super::routes())]);
        (client, db)
    }

    fn user_id_cookie(response: &LocalResponse) -> Option<Cookie<'static>> {
        let cookie = response
            .headers()
            .get("Set-Cookie")
            .find(|v| v.starts_with(COOKIE_EMAIL_KEY))
            .and_then(|val| Cookie::parse_encoded(val).ok());
        cookie.map(|c| c.into_owned())
    }

    #[test]
    fn get_login() {
        let (client, _) = setup();
        let res = client.get("/login").dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        assert!(user_id_cookie(&res).is_none());
        let body_str = res.into_string().unwrap();
        assert!(body_str.contains("action=\"login\""));
    }

    #[test]
    fn post_login_fails() {
        let (client, pool) = setup();
        register_user(&pool, "foo@bar.com", "bazbaz", true);
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=foo%40bar.com&password=invalid")
            .dispatch();
        assert_eq!(res.status(), HttpStatus::SeeOther);
        for h in res.headers().iter() {
            match h.name.as_str() {
                "Location" => assert_eq!(h.value, "/login"),
                "Content-Length" => assert_eq!(h.value.parse::<i32>().unwrap(), 0),
                _ => { /* let these through */ }
            }
        }
    }

    #[test]
    fn post_login_success() {
        let (client, pool) = setup();
        register_user(&pool, "foo@bar.com", "baz baz", true);
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=foo%40bar.com&password=baz baz")
            .dispatch();
        assert_eq!(res.status(), HttpStatus::SeeOther);
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
