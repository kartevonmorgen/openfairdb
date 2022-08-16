use std::collections::HashMap;

use ::captcha::{gen, Difficulty};
use parking_lot::{Mutex, MutexGuard};
use rocket::{
    data::{Data, ToByteUnit},
    get,
    http::{ContentType, Cookie, CookieJar, Status},
    post, State,
};
use uuid::Uuid;

use super::super::guards::{COOKIE_CAPTCHA_KEY, MAX_CAPTCHA_TTL};

pub struct CaptchaCache(Mutex<HashMap<Uuid, Option<String>>>);

impl CaptchaCache {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
    pub fn prepare(&self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.lock().insert(uuid, None);
        uuid
    }
    pub fn is_prepared(&self, uuid: &Uuid) -> bool {
        self.lock().get(uuid) == Some(&None)
    }
    pub fn activate(&self, uuid: Uuid, answer: String) {
        self.lock().insert(uuid, Some(answer));
    }
    pub fn verify(&self, uuid: Uuid, answer: String) -> bool {
        let is_valid = self.lock().get(&uuid) == Some(&Some(answer));
        self.lock().remove(&uuid);
        is_valid
    }
    fn lock(&self) -> MutexGuard<HashMap<Uuid, Option<String>>> {
        self.0.lock()
    }
}

#[post("/captcha", rank = 2)]
pub fn post_captcha(captcha_cache: &State<CaptchaCache>) -> Result<String, Status> {
    let uuid = captcha_cache.prepare();
    Ok(uuid.simple().to_string())
}

#[get("/captcha/<token>")]
pub fn get_captcha(
    captcha_cache: &State<CaptchaCache>,
    token: &str,
) -> Result<(ContentType, Vec<u8>), Status> {
    let uuid: Uuid = token.parse().map_err(|_| Status::BadRequest)?;
    if !captcha_cache.is_prepared(&uuid) {
        return Err(Status::BadRequest);
    }
    let captcha = gen(Difficulty::Easy);
    let answer = captcha.chars_as_string();
    captcha_cache.activate(uuid, answer);
    let png = captcha.as_png().ok_or(Status::InternalServerError)?;

    Ok((ContentType::PNG, png))
}

#[post("/captcha/<token>/verify", format = "plain", data = "<data>")]
pub async fn post_captcha_verify(
    cookies: &CookieJar<'_>,
    captcha_cache: &State<CaptchaCache>,
    token: &str,
    data: Data<'_>,
) -> Result<(), Status> {
    let answer = data
        .open(36.bytes())
        .into_string()
        .await
        .map_err(|_| Status::BadRequest)?
        .into_inner();
    let token: Uuid = token.parse().map_err(|_| Status::BadRequest)?;
    if captcha_cache.verify(token, answer) {
        let now_utc = time::OffsetDateTime::now_utc();
        let expires = now_utc + MAX_CAPTCHA_TTL;
        let cookie_value = now_utc.unix_timestamp().to_string();
        cookies.add_private(
            Cookie::build(COOKIE_CAPTCHA_KEY, cookie_value)
                .expires(expires)
                .same_site(rocket::http::SameSite::None)
                .finish(),
        );
        Ok(())
    } else {
        Err(Status::BadRequest)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use uuid::Uuid;

    use super::{
        super::{super::guards::COOKIE_CAPTCHA_KEY, tests::prelude::*},
        *,
    };

    #[test]
    fn request_new_captcha_challenge() {
        let (client, _) = setup();
        let res = client.post("/captcha").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.into_string().unwrap();
        assert!(Uuid::from_str(&body_str).is_ok());
    }

    #[test]
    fn request_captcha_image() {
        let (client, _) = setup();
        let res = client.post("/captcha").dispatch();
        let token = res.into_string().unwrap();
        let res = client.get(format!("/captcha/{}", token)).dispatch();
        assert_eq!(res.status(), Status::Ok);
    }

    #[test]
    fn request_invalid_captcha_image() {
        let (client, _) = setup();
        let token = Uuid::new_v4().simple().to_string();
        let res = client.get(format!("/captcha/{}", token)).dispatch();
        assert_eq!(res.status(), Status::BadRequest);
    }

    #[test]
    fn verify_invalid_captcha_answer() {
        let (client, _) = setup();
        let token = Uuid::new_v4().simple().to_string();
        let res = client
            .post(format!("/captcha/{}/verify", token))
            .header(ContentType::Plain)
            .body("foo")
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);
    }

    #[test]
    fn verify_valid_captcha_answer() {
        let (client, _) = setup();
        let res = client.post("/captcha").dispatch();
        let token_str = res.into_string().unwrap();
        let _ = client.get(format!("/captcha/{}", token_str)).dispatch();
        let cache = client.rocket().state::<CaptchaCache>().unwrap();
        let uuid = Uuid::from_str(&token_str).unwrap();
        let answer = cache.lock().get(&uuid).unwrap().clone().unwrap();
        let res = client
            .post(format!("/captcha/{}/verify", token_str))
            .header(ContentType::Plain)
            .body(answer)
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(cookie_from_response(&res, COOKIE_CAPTCHA_KEY).is_some());
        let cache = client.rocket().state::<CaptchaCache>().unwrap();
        assert!(cache.lock().get(&uuid).is_none());
    }

    pub fn get_valid_captcha_cookie(client: &Client) -> Option<Cookie<'static>> {
        let res = client.post("/captcha").dispatch();
        let token_str = res.into_string().unwrap();
        let _ = client.get(format!("/captcha/{}", token_str)).dispatch();
        let cache = client.rocket().state::<CaptchaCache>().unwrap();
        let uuid = Uuid::from_str(&token_str).unwrap();
        let answer = cache.lock().get(&uuid).unwrap().clone().unwrap();
        let res = client
            .post(format!("/captcha/{}/verify", token_str))
            .header(ContentType::Plain)
            .body(answer)
            .dispatch();
        cookie_from_response(&res, COOKIE_CAPTCHA_KEY)
    }
}
