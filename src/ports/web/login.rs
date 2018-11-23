use crate::core::{prelude::*, usecases};
use pwhash::bcrypt;
use rocket::{
    http::{Cookie, Cookies},
    outcome::IntoOutcome,
    request::{self, FromRequest, Request},
};

#[derive(Debug)]
pub struct UserLogin(pub String);

#[derive(Debug)]
pub struct AdminLogin(pub String);

const COOKIE_USER_KEY: &str = "user_id";
const COOKIE_ADMIN_KEY: &str = "admin_id";

impl<'a, 'r> FromRequest<'a, 'r> for UserLogin {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<UserLogin, !> {
        request
            .cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(UserLogin)
            .or_forward(())
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AdminLogin {
    type Error = !;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AdminLogin, !> {
        request
            .cookies()
            .get_private(COOKIE_ADMIN_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(AdminLogin)
            .or_forward(())
    }
}

pub fn login(db: &impl Db, cookies: &mut Cookies, data: usecases::Credentials) -> Result<()> {
    let (username, access) = usecases::login(&*db, &data)?;
    debug!("User {} logged in as {:?}", username, access);
    let key = match access {
        AccessLevel::Admin => COOKIE_ADMIN_KEY,
        _ => COOKIE_USER_KEY,
    };
    cookies.add_private(Cookie::new(key, username));
    Ok(())
}

pub fn logout(cookies: &mut Cookies) {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    cookies.remove_private(Cookie::named(COOKIE_ADMIN_KEY));
}

#[cfg(test)]
mod tests {
    use super::super::sqlite;
    use super::*;
    use rocket::local::Client;

    fn setup() -> (Client, sqlite::ConnectionPool) {
        let (client, db) = super::super::tests::setup();
        let mut conn = db.get().unwrap();
        let users = vec![
            User {
                id: "user".into(),
                username: "user".into(),
                password: bcrypt::hash("user").unwrap(),
                email: "user@domain".into(),
                email_confirmed: true,
                access: AccessLevel::User,
            },
            User {
                id: "admin".into(),
                username: "admin".into(),
                password: bcrypt::hash("admin").unwrap(),
                email: "admin@domain".into(),
                email_confirmed: true,
                access: AccessLevel::Admin,
            },
        ];
        for u in users {
            conn.create_user(&u).unwrap();
        }
        (client, db)
    }

    #[test]
    fn user_login() {
        let (client, db_pool) = setup();
        let req = client.get("/foo");
        let mut cookies = req.inner().cookies();
        let conn = db_pool.get().unwrap();
        login(
            &*conn,
            &mut cookies,
            usecases::Credentials {
                username: "user".into(),
                password: "user".into(),
            },
        )
        .unwrap();
        assert!(cookies.get_private(COOKIE_USER_KEY).is_some());
        assert!(cookies.get_private(COOKIE_ADMIN_KEY).is_none());
    }

    #[test]
    fn admin_login() {
        let (client, db_pool) = setup();
        let req = client.get("/foo");
        let mut cookies = req.inner().cookies();
        let conn = db_pool.get().unwrap();
        login(
            &*conn,
            &mut cookies,
            usecases::Credentials {
                username: "admin".into(),
                password: "admin".into(),
            },
        )
        .unwrap();
        assert!(cookies.get_private(COOKIE_ADMIN_KEY).is_some());
        assert!(cookies.get_private(COOKIE_USER_KEY).is_none());
    }

    #[test]
    fn user_login_fail() {
        let (client, db_pool) = setup();
        let req = client.get("/foo");
        let mut cookies = req.inner().cookies();
        let conn = db_pool.get().unwrap();
        assert!(login(
            &*conn,
            &mut cookies,
            usecases::Credentials {
                username: "user".into(),
                password: "invalid".into()
            }
        )
        .is_err());
        assert!(cookies.get_private(COOKIE_USER_KEY).is_none());
    }

    #[test]
    fn remove_cookies_on_logout() {
        let (client, _) = setup();
        let req_0 = client
            .get("/foo")
            .private_cookie(Cookie::new(COOKIE_USER_KEY, "bar"));
        let req_1 = client
            .get("/foo")
            .private_cookie(Cookie::new(COOKIE_ADMIN_KEY, "baz"));
        let mut cookies_0 = req_0.inner().cookies();
        let mut cookies_1 = req_1.inner().cookies();
        assert!(cookies_0.get_private(COOKIE_USER_KEY).is_some());
        assert!(cookies_1.get_private(COOKIE_ADMIN_KEY).is_some());
        logout(&mut cookies_0);
        logout(&mut cookies_1);
        assert!(cookies_0.get_private(COOKIE_USER_KEY).is_none());
        assert!(cookies_1.get_private(COOKIE_ADMIN_KEY).is_none());
    }
}
