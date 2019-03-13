use super::*;

#[post("/users", format = "application/json", data = "<u>")]
pub fn post_user(db: sqlite::Connections, u: Json<usecases::NewUser>) -> Result<()> {
    let new_user = u.into_inner();
    let user = {
        let mut db = db.exclusive()?;
        usecases::create_new_user(&mut *db, new_user.clone())?;
        db.get_user(&new_user.username)?
    };

    notify::user_registered_kvm(&user);

    Ok(Json(()))
}

#[post(
    "/users/request_password_reset",
    format = "application/json",
    data = "<data>"
)]
pub fn post_request_password_reset(
    connections: sqlite::Connections,
    data: Json<json::RequestPasswordReset>,
) -> Result<()> {
    let req = data.into_inner();

    flows::request_password_reset(&connections, &req.email_or_username)?;

    Ok(Json(()))
}

#[post("/users/reset_password", format = "application/json", data = "<data>")]
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
