use super::*;

#[post("/users", format = "application/json", data = "<u>")]
pub fn post_user(db: sqlite::Connections, u: Json<usecases::NewUser>) -> Result<()> {
    let new_user = u.into_inner();
    let user = {
        let db = db.exclusive()?;
        usecases::create_new_user(&*db, new_user.clone())?;
        db.get_user(&new_user.username)?
    };

    notify::user_registered_kvm(&user);

    Ok(Json(()))
}

#[post(
    "/users/reset-password-request",
    format = "application/json",
    data = "<data>"
)]
pub fn post_request_password_reset(
    connections: sqlite::Connections,
    data: Json<json::RequestPasswordReset>,
) -> Result<()> {
    let req = data.into_inner();

    flows::reset_password_request(&connections, &req.email_or_username)?;

    Ok(Json(()))
}

#[post("/users/reset-password", format = "application/json", data = "<data>")]
pub fn post_reset_password(
    connections: sqlite::Connections,
    data: Json<json::ResetPassword>,
) -> Result<()> {
    let req = data.into_inner();

    let token = EmailToken::decode_from_str(&req.token)?;
    let new_password = req.new_password.parse::<Password>()?;
    flows::reset_password_with_email_token(
        &connections,
        &req.email_or_username,
        token,
        new_password,
    )?;

    Ok(Json(()))
}

#[delete("/users/<u_id>")]
pub fn delete_user(db: sqlite::Connections, user: Login, u_id: String) -> Result<()> {
    usecases::delete_user(&mut *db.exclusive()?, &user.0, &u_id)?;
    Ok(Json(()))
}

#[get("/users/<username>", format = "application/json")]
pub fn get_user(db: sqlite::Connections, user: Login, username: String) -> Result<json::User> {
    let (_, email) = usecases::get_user(&*db.shared()?, &user.0, &username)?;
    Ok(Json(json::User { username, email }))
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
            .body(r#"{"email_or_username":"user@example.com"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // User gets an email with the corresponding token
        let token = db
            .shared()
            .unwrap()
            .get_email_token_credentials_by_email_or_username("user@example.com")
            .unwrap()
            .token
            .encode_to_string();

        // User send the new password to the server
        let res = client
           .post("/users/reset-password")
           .header(ContentType::JSON)
           .body(format!(r#"{{"email_or_username":"user@example.com","token":"{}","new_password":"12345678"}}"#, token))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        // User can't login with old password
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"username":"userexamplecom","password":"secret"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Unauthorized);

        // User can login with the new password
        let res = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"username":"userexamplecom","password":"12345678"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
    }
}
