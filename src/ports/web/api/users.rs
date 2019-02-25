use super::*;

#[post("/users", format = "application/json", data = "<u>")]
pub fn post_user(db: sqlite::Connections, u: Json<usecases::NewUser>) -> Result<()> {
    let new_user = u.into_inner();
    let user = {
        let mut db = db.exclusive()?;
        usecases::create_new_user(&mut *db, new_user.clone())?;
        db.get_user(&new_user.username)?
    };
    let subject = "Karte von morgen: bitte bestätige deine Email-Adresse";
    let body = user_communication::email_confirmation_email(&user.id);

    #[cfg(feature = "email")]
    notify::send_mails(&[user.email], subject, &body);

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
