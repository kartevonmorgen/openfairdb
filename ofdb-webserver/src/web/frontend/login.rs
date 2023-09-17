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

use crate::{core::usecases, web::sqlite::Connections};
use ofdb_core::{entities::EmailAddress, usecases::Error as ParameterError};

#[derive(FromForm)]
pub struct LoginCredentials<'r> {
    pub(crate) email: &'r str,
    pub(crate) password: &'r str,
}

#[allow(clippy::result_large_err)]
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

#[allow(clippy::result_large_err)]
#[post("/login", data = "<credentials>")]
pub fn post_login(
    db: Connections,
    credentials: Form<LoginCredentials>,
    cookies: &CookieJar<'_>,
) -> std::result::Result<Redirect, Flash<Redirect>> {
    let Ok(db) = db.shared() else {
        return Err(Flash::error(
            Redirect::to(uri!(get_login)),
            "We are so sorry! An internal server error has occurred. Please try again later.",
        ));
    };
    let Ok(email) = credentials.email.parse::<EmailAddress>() else {
        return Err(Flash::error(
            Redirect::to(uri!(get_login)),
            "Invalid email or password.",
        ));
    };
    let login = usecases::Credentials {
        email: &email,
        password: credentials.password,
    };
    match usecases::login_with_email(&db, &login) {
        Err(err) => {
            let msg = match err {
                ParameterError::EmailNotConfirmed => {
                    "You have to confirm your email address first."
                }
                ParameterError::Credentials => "Invalid email or password.",
                _ => panic!(),
            };
            Err(Flash::error(Redirect::to(uri!(get_login)), msg))
        }
        Ok(_) => {
            let email = login.email.to_string();
            cookies.add_private(
                Cookie::build(COOKIE_EMAIL_KEY, email)
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .finish(),
            );
            Ok(Redirect::to(uri!(super::get_index)))
        }
    }
}

#[post("/logout")]
pub fn post_logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
    Flash::success(
        Redirect::to(uri!(super::get_index)),
        "You have successfully logged out.",
    )
}

#[cfg(test)]
pub mod tests {
    use rocket::http::Status as HttpStatus;

    use super::*;
    use crate::web::{
        self,
        tests::{prelude::*, register_user},
    };

    fn setup() -> (Client, Connections) {
        let (client, db, _) = web::tests::rocket_test_setup(vec![("/", super::super::routes())]);
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
