use super::*;
use crate::adapters::json::from_json;
use ofdb_boundary::NewUser;

#[post("/users", format = "application/json", data = "<u>")]
pub fn post_user(db: sqlite::Connections, n: Notify, u: JsonResult<NewUser>) -> Result<()> {
    let new_user = from_json::new_user(u?.into_inner());
    let user = {
        let db = db.exclusive()?;
        usecases::create_new_user(&db, new_user.clone())?;
        db.get_user_by_email(&new_user.email)?
    };
    n.user_registered_kvm(&user);
    Ok(Json(()))
}

#[post(
    "/users/reset-password-request",
    format = "application/json",
    data = "<data>"
)]
pub fn post_request_password_reset(
    connections: sqlite::Connections,
    notify: Notify,
    data: JsonResult<json::RequestPasswordReset>,
) -> Result<()> {
    let req = data?.into_inner();
    flows::reset_password_request(&connections, &*notify, &req.email)?;

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
    usecases::delete_user(&db.exclusive()?, account.email(), &email)?;
    Ok(Json(()))
}

#[get("/users/current", format = "application/json")]
pub fn get_current_user(db: sqlite::Connections, account: Account) -> Result<json::User> {
    let user = usecases::get_user(&db.shared()?, account.email(), account.email())?;
    Ok(Json(user.into()))
}

#[get("/users/<email>", format = "application/json", rank = 2)]
pub fn get_user(db: sqlite::Connections, account: Account, email: String) -> Result<json::User> {
    let user = usecases::get_user(&db.shared()?, account.email(), &email)?;
    Ok(Json(user.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::web::{api::tests::prelude::*, tests::register_user};

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
            .get_user_token_by_email("user@example.com")
            .unwrap()
            .email_nonce
            .encode_to_string();
        assert_eq!(
            "user@example.com",
            EmailNonce::decode_from_str(&token).unwrap().email
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

        let email = Email::from("user@example.com");
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
        assert_eq!(email, current_user.email.into());
        assert_eq!(email_confirmed, current_user.email_confirmed);
        assert_eq!(Role::User, current_user.role.into());
    }
}
