use super::*;
use crate::adapters::json::from_json;
use ofdb_boundary::NewUser;
use ofdb_core::gateways::notify::NotificationEvent;

#[post("/login", format = "application/json", data = "<login>")]
pub fn post_login(
    db: sqlite::Connections,
    cookies: &CookieJar<'_>,
    login: JsonResult<json::Credentials>,
    jwt_state: &State<jwt::JwtState>,
) -> Result<Option<ofdb_boundary::JwtToken>> {
    let login = login?.into_inner();
    {
        let credentials = usecases::Credentials {
            email: &login.email.parse()?,
            password: &login.password,
        };
        usecases::login_with_email(&db.shared()?, &credentials).map_err(|err| {
            log::debug!("Login with email '{}' failed: {}", login.email, err);
            err
        })?;
    }

    let mut response = None;
    if cfg!(feature = "jwt") {
        let token = jwt_state.generate_token(&login.email)?;
        response = Some(ofdb_boundary::JwtToken { token });
    }
    if cfg!(feature = "cookies") {
        cookies.add_private(
            Cookie::build(COOKIE_EMAIL_KEY, login.email)
                .same_site(rocket::http::SameSite::None)
                .finish(),
        );
    }
    Ok(Json(response))
}

#[post("/logout", format = "application/json")]
pub fn post_logout(
    auth: Auth,
    cookies: &CookieJar<'_>,
    jwt_state: &State<jwt::JwtState>,
) -> Json<()> {
    cookies.remove_private(Cookie::named(COOKIE_EMAIL_KEY));
    if cfg!(feature = "jwt") {
        for bearer in auth.bearer_tokens() {
            jwt_state.blacklist_token(bearer.to_owned());
        }
    }
    Json(())
}

#[post("/users", format = "application/json", data = "<new_user>")]
pub fn post_user(
    db: sqlite::Connections,
    notify: &State<Notify>,
    new_user: JsonResult<NewUser>,
) -> Result<()> {
    let new_user = from_json::try_new_user(new_user?.into_inner())?;
    // TODO: move this into ofdb-application
    let user = {
        let db = db.exclusive()?;
        usecases::create_new_user(&db, new_user.clone())?;
        db.get_user_by_email(&new_user.email)?
    };
    let token = EmailNonce {
        email: user.email.clone(),
        nonce: Nonce::new(),
    }
    .encode_to_string();
    let confirmation_url = format!("https://kartevonmorgen.org/#/?confirm_email={}", token)
        .parse()
        .expect("Valid email confirmation URL");
    notify.notify(NotificationEvent::UserRegistered {
        user: &user,
        confirmation_url,
    });
    Ok(Json(()))
}

#[post(
    "/users/reset-password-request",
    format = "application/json",
    data = "<data>"
)]
pub fn post_request_password_reset(
    connections: sqlite::Connections,
    notify: &State<Notify>,
    data: JsonResult<json::RequestPasswordReset>,
) -> Result<()> {
    let req = data?.into_inner();
    flows::reset_password_request(&connections, &*notify.0, &req.email.parse()?)?;

    Ok(Json(()))
}

#[post("/users/reset-password", format = "application/json", data = "<data>")]
pub fn post_reset_password(
    connections: sqlite::Connections,
    data: JsonResult<json::ResetPassword>,
) -> Result<()> {
    let req = data?.into_inner();

    let email_nonce = EmailNonce::decode_from_str(&req.token)?;
    let new_password = req.new_password.parse::<Password>()?;
    flows::reset_password_with_email_nonce(&connections, email_nonce, new_password)?;

    Ok(Json(()))
}

#[delete("/users/<email>")]
pub fn delete_user(db: sqlite::Connections, account: Account, email: String) -> Result<()> {
    usecases::delete_user(&db.exclusive()?, account.email(), &email.parse()?)?;
    Ok(Json(()))
}

#[get("/users/current", format = "application/json")]
pub fn get_current_user(db: sqlite::Connections, account: Account) -> Result<json::User> {
    let user = usecases::get_user(&db.shared()?, account.email(), account.email())?;
    Ok(Json(user.into()))
}

#[get("/users/<email>", format = "application/json", rank = 2)]
pub fn get_user(db: sqlite::Connections, account: Account, email: String) -> Result<json::User> {
    let user = usecases::get_user(&db.shared()?, account.email(), &email.parse()?)?;
    Ok(Json(user.into()))
}

#[post(
    "/confirm-email-address",
    format = "application/json",
    data = "<token>"
)]
pub fn confirm_email_address(
    db: sqlite::Connections,
    token: JsonResult<json::ConfirmEmailAddress>,
) -> Result<()> {
    let token = token?.into_inner().token;
    usecases::confirm_email_address(&db.exclusive()?, &token)?;
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::{api::tests::prelude::*, tests::register_user};

    #[test]
    fn reset_password() {
        let (client, db) = setup();
        register_user(&db, "user@example.com", "secret", true);

        // User sends the request
        let res = client
            .post("/users/reset-password-request")
            .header(ContentType::JSON)
            .body(r#"{"email":"user@example.com"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // User gets an email with the corresponding token
        let token = db
            .shared()
            .unwrap()
            .get_user_token_by_email(&"user@example.com".parse::<EmailAddress>().unwrap())
            .unwrap()
            .email_nonce
            .encode_to_string();
        assert_eq!(
            "user@example.com",
            EmailNonce::decode_from_str(&token).unwrap().email.as_str()
        );

        // User send the new password to the server
        let res = client
            .post("/users/reset-password")
            .header(ContentType::JSON)
            .body(format!(
                "{{\"token\":\"{}\",\"new_password\":\"12345678\"}}",
                token
            ))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // User can't login with old password
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email":"user@example.com","password":"secret"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Unauthorized);

        // User can login with the new password
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email":"user@example.com","password":"12345678"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
    }

    #[test]
    fn current_user() {
        let (client, db) = setup();

        let email = "user@example.com".parse::<EmailAddress>().unwrap();
        let email_confirmed = true;

        register_user(&db, email.as_str(), "secret", email_confirmed);

        // Before login
        let res = client
            .get("/users/current")
            .header(ContentType::JSON)
            .dispatch();
        assert_eq!(res.status(), Status::Unauthorized);

        // Login
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email":"user@example.com","password":"secret"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // After login
        let res = client
            .get("/users/current")
            .header(ContentType::JSON)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_string().unwrap();
        let current_user: json::User = serde_json::from_str(&body).unwrap();
        assert_eq!(email.as_str(), current_user.email);
        assert_eq!(email_confirmed, current_user.email_confirmed);
        assert_eq!(Role::User, current_user.role.into());
    }
}
