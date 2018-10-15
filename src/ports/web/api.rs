use super::{sqlite::DbConn, util};
use adapters::{self, json, user_communication};
use core::{
    prelude::*,
    usecases::{self, DuplicateType},
    util::geo,
};
use csv;
use infrastructure::error::AppError;
use rocket::{
    self,
    http::{ContentType, Cookie, Cookies, Status},
    request::{self, FromRequest, Request},
    response::{content::Content, Responder, Response},
    Outcome, Route,
};
use rocket_contrib::Json;
use serde_json::ser::to_string;
use std::result;

type Result<T> = result::Result<Json<T>, AppError>;

const COOKIE_USER_KEY: &str = "user_id";

impl<'a, 'r> FromRequest<'a, 'r> for Login {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Login, ()> {
        let user = request
            .cookies()
            .get_private(COOKIE_USER_KEY)
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Login);
        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        login,
        logout,
        delete_user,
        confirm_email_address,
        subscribe_to_bbox,
        get_bbox_subscriptions,
        unsubscribe_all_bboxes,
        get_entry,
        post_entry,
        post_user,
        post_rating,
        put_entry,
        get_user,
        get_categories,
        get_tags,
        get_ratings,
        get_category,
        get_search,
        get_duplicates,
        get_count_entries,
        get_count_tags,
        get_version,
        csv_export
    ]
}

#[derive(FromForm, Clone)]
struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    text: Option<String>,
    tags: Option<String>,
}

#[get("/search?<search>")]
fn get_search(db: DbConn, search: SearchQuery) -> Result<json::SearchResponse> {
    let bbox = geo::extract_bbox(&search.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let categories = match search.categories {
        Some(cat_str) => Some(util::extract_ids(&cat_str)),
        None => None,
    };

    let mut tags = vec![];

    if let Some(ref txt) = search.text {
        tags = util::extract_hash_tags(txt);
    }

    if let Some(tags_str) = search.tags {
        for t in util::extract_ids(&tags_str) {
            tags.push(t);
        }
    }

    let text = match search.text {
        Some(txt) => util::remove_hash_tags(&txt),
        None => "".into(),
    };

    let avg_ratings = match super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let req = usecases::SearchRequest {
        bbox,
        categories,
        text,
        tags,
        entry_ratings: &*avg_ratings,
    };

    let (visible, invisible) = usecases::search(&*db, &req)?;

    let visible = visible
        .into_iter()
        .map(|e| json::EntryIdWithCoordinates {
            id: e.id,
            lat: e.lat,
            lng: e.lng,
        })
        .collect();

    let invisible = invisible
        .into_iter()
        .map(|e| json::EntryIdWithCoordinates {
            id: e.id,
            lat: e.lat,
            lng: e.lng,
        })
        .collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}

#[derive(Deserialize, Debug, Clone)]
struct Login(String);

#[derive(Deserialize, Debug, Clone)]
struct UserId {
    u_id: String,
}

#[get("/entries/<ids>")]
fn get_entry(db: DbConn, ids: String) -> Result<Vec<json::Entry>> {
    let ids = util::extract_ids(&ids);
    let entries = usecases::get_entries(&*db, &ids)?;
    let ratings = usecases::get_ratings_by_entry_ids(&*db, &ids)?;
    Ok(Json(
        entries
            .into_iter()
            .map(|e| {
                let r = ratings.get(&e.id).cloned().unwrap_or_else(|| vec![]);
                json::Entry::from_entry_with_ratings(e, r)
            })
            .collect::<Vec<json::Entry>>(),
    ))
}

#[get("/duplicates")]
fn get_duplicates(db: DbConn) -> Result<Vec<(String, String, DuplicateType)>> {
    let entries = db.all_entries()?;
    let ids = usecases::find_duplicates(&entries);
    Ok(Json(ids))
}

#[get("/count/entries")]
fn get_count_entries(db: DbConn) -> Result<usize> {
    let entries = db.all_entries()?;
    Ok(Json(entries.len()))
}

#[get("/count/tags")]
fn get_count_tags(db: DbConn) -> Result<usize> {
    Ok(Json(db.all_tags()?.len()))
}

#[get("/server/version")]
fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[post("/users", format = "application/json", data = "<u>")]
fn post_user(mut db: DbConn, u: Json<usecases::NewUser>) -> Result<()> {
    let new_user = u.into_inner();
    usecases::create_new_user(&mut *db, new_user.clone())?;
    let user = db.get_user(&new_user.username)?;
    let subject = "Karte von Morgen: bitte bestätige deine Email-Adresse";
    let body = user_communication::email_confirmation_email(&user.id);

    #[cfg(feature = "email")]
    util::send_mails(&[user.email], subject, &body);

    Ok(Json(()))
}

#[delete("/users/<u_id>")]
fn delete_user(mut db: DbConn, user: Login, u_id: String) -> Result<()> {
    usecases::delete_user(&mut *db, &user.0, &u_id)?;
    Ok(Json(()))
}

#[post("/ratings", format = "application/json", data = "<u>")]
fn post_rating(mut db: DbConn, u: Json<usecases::RateEntry>) -> Result<()> {
    let u = u.into_inner();
    let e_id = u.entry.clone();
    usecases::rate_entry(&mut *db, u)?;
    super::calculate_rating_for_entry(&*db, &e_id)?;
    Ok(Json(()))
}

#[get("/ratings/<id>")]
fn get_ratings(db: DbConn, id: String) -> Result<Vec<json::Rating>> {
    let ratings = usecases::get_ratings(&*db, &util::extract_ids(&id))?;
    let r_ids: Vec<String> = ratings.iter().map(|r| r.id.clone()).collect();
    let comments = usecases::get_comments_by_rating_ids(&*db, &r_ids)?;
    let result = ratings
        .into_iter()
        .map(|x| json::Rating {
            id: x.id.clone(),
            created: x.created,
            title: x.title,
            value: x.value,
            context: x.context,
            source: x.source.unwrap_or_else(|| "".into()),
            comments: comments
                .get(&x.id)
                .cloned()
                .unwrap_or_else(|| vec![])
                .into_iter()
                .map(|c| json::Comment {
                    id: c.id.clone(),
                    created: c.created,
                    text: c.text,
                })
                .collect(),
        })
        .collect();
    Ok(Json(result))
}

#[post("/login", format = "application/json", data = "<login>")]
fn login(mut db: DbConn, mut cookies: Cookies, login: Json<usecases::Login>) -> Result<()> {
    let username = usecases::login(&mut *db, &login.into_inner())?;
    cookies.add_private(Cookie::new(COOKIE_USER_KEY, username));
    Ok(Json(()))
}

#[post("/logout", format = "application/json")]
fn logout(mut cookies: Cookies) -> Result<()> {
    cookies.remove_private(Cookie::named(COOKIE_USER_KEY));
    Ok(Json(()))
}

#[post(
    "/confirm-email-address",
    format = "application/json",
    data = "<user>"
)]
fn confirm_email_address(mut db: DbConn, user: Json<UserId>) -> Result<()> {
    let u_id = user.into_inner().u_id;
    let u = db.confirm_email_address(&u_id)?;
    if u.id == u_id {
        Ok(Json(()))
    } else {
        Err(AppError::Business(Error::Repo(RepoError::NotFound)))
    }
}

#[post(
    "/subscribe-to-bbox",
    format = "application/json",
    data = "<coordinates>"
)]
fn subscribe_to_bbox(
    mut db: DbConn,
    user: Login,
    coordinates: Json<Vec<Coordinate>>,
) -> Result<()> {
    let coordinates = coordinates.into_inner();
    let Login(username) = user;
    usecases::subscribe_to_bbox(&coordinates, &username, &mut *db)?;
    Ok(Json(()))
}

#[delete("/unsubscribe-all-bboxes")]
fn unsubscribe_all_bboxes(mut db: DbConn, user: Login) -> Result<()> {
    let Login(username) = user;
    usecases::unsubscribe_all_bboxes_by_username(&mut *db, &username)?;
    Ok(Json(()))
}

#[get("/bbox-subscriptions")]
fn get_bbox_subscriptions(db: DbConn, user: Login) -> Result<Vec<json::BboxSubscription>> {
    let Login(username) = user;
    let user_subscriptions = usecases::get_bbox_subscriptions(&username, &*db)?
        .into_iter()
        .map(|s| json::BboxSubscription {
            id: s.id,
            south_west_lat: s.bbox.south_west.lat,
            south_west_lng: s.bbox.south_west.lng,
            north_east_lat: s.bbox.north_east.lat,
            north_east_lng: s.bbox.north_east.lng,
        })
        .collect();
    Ok(Json(user_subscriptions))
}

#[get("/users/<username>", format = "application/json")]
fn get_user(mut db: DbConn, user: Login, username: String) -> Result<json::User> {
    let (_, email) = usecases::get_user(&mut *db, &user.0, &username)?;
    Ok(Json(json::User { username, email }))
}

#[post("/entries", format = "application/json", data = "<e>")]
fn post_entry(mut db: DbConn, e: Json<usecases::NewEntry>) -> Result<String> {
    let e = e.into_inner();
    let id = usecases::create_new_entry(&mut *db, e.clone())?;
    let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
    let all_categories = db.all_categories()?;
    util::notify_create_entry(&email_addresses, &e, &id, all_categories);
    Ok(Json(id))
}

#[put("/entries/<id>", format = "application/json", data = "<e>")]
fn put_entry(mut db: DbConn, id: String, e: Json<usecases::UpdateEntry>) -> Result<String> {
    let e = e.into_inner();
    usecases::update_entry(&mut *db, e.clone())?;
    let email_addresses = usecases::email_addresses_by_coordinate(&mut *db, &e.lat, &e.lng)?;
    let all_categories = db.all_categories()?;
    util::notify_update_entry(&email_addresses, &e, all_categories);
    Ok(Json(id))
}

#[get("/tags")]
fn get_tags(db: DbConn) -> Result<Vec<String>> {
    Ok(Json(db.all_tags()?.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
fn get_categories(db: DbConn) -> Result<Vec<Category>> {
    let categories = db.all_categories()?;
    Ok(Json(categories))
}

#[get("/categories/<id>")]
fn get_category(db: DbConn, id: String) -> Result<String> {
    let ids = util::extract_ids(&id);
    let categories = db.all_categories()?;
    let res = match ids.len() {
        0 => to_string(&categories),

        1 => {
            let id = ids[0].clone();
            let e = categories
                .into_iter()
                .find(|x| x.id == id)
                .ok_or(RepoError::NotFound)?;
            to_string(&e)
        }
        _ => to_string(
            &categories
                .into_iter()
                .filter(|e| ids.iter().any(|id| e.id == id.clone()))
                .collect::<Vec<Category>>(),
        ),
    }?;
    Ok(Json(res))
}

#[derive(FromForm, Clone, Serialize)]
struct CsvExport {
    bbox: String,
}

#[get("/export/entries.csv?<export>")]
fn csv_export<'a>(db: DbConn, export: CsvExport) -> result::Result<Content<String>, AppError> {
    let bbox = geo::extract_bbox(&export.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let entries: Vec<_> = db.get_entries_by_bbox(&bbox)?;
    let all_categories: Vec<_> = db.all_categories()?;
    let avg_ratings = match super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let entries_categories_and_ratings = entries
        .into_iter()
        .map(|e| {
            let categories = all_categories
                .iter()
                .filter(|c1| e.categories.iter().any(|c2| *c2 == c1.id))
                .cloned()
                .collect::<Vec<Category>>();
            let avg_rating = *avg_ratings.get(&e.id).unwrap_or_else(|| &0.0);
            (e, categories, avg_rating)
        })
        .collect::<Vec<_>>();

    let records: Vec<adapters::csv::CsvRecord> = entries_categories_and_ratings
        .into_iter()
        .map(adapters::csv::CsvRecord::from)
        .collect();

    let buff: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buff);

    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(Content(ContentType::CSV, data))
}

impl<'r> Responder<'r> for AppError {
    fn respond_to(self, _: &rocket::Request) -> result::Result<Response<'r>, Status> {
        if let AppError::Business(ref err) = self {
            match *err {
                Error::Parameter(ref err) => {
                    return Err(match *err {
                        ParameterError::Credentials => Status::Unauthorized,
                        ParameterError::UserExists => <Status>::new(400, "UserExists"),
                        ParameterError::EmailNotConfirmed => {
                            <Status>::new(403, "EmailNotConfirmed")
                        }
                        ParameterError::Forbidden => Status::Forbidden,
                        _ => Status::BadRequest,
                    })
                }
                Error::Repo(ref err) => {
                    if let RepoError::NotFound = *err {
                        return Err(Status::NotFound);
                    }
                }
                _ => {}
            }
        }
        error!("Error: {}", self);
        Err(Status::InternalServerError)
    }
}

#[cfg(test)]
mod tests {
    //use super::super::mockdb;

    //fn setup() -> mockdb::ConnectionPool {
    //    mockdb::create_connection_pool(":memory:").unwrap()
    //}

    //use super::super::{mockdb::DbConn, ENTRY_RATINGS};
    //use test::Bencher;

    //#[ignore]
    //#[bench]
    //fn bench_search_in_10_000_rated_entries(b: &mut Bencher) {
    //    let (entries, ratings) = ::core::util::sort::tests::create_entries_with_ratings(10_000);
    //    let pool = setup();
    //    let mut conn = pool.get().unwrap();
    //    conn.entries = entries;
    //    conn.ratings = ratings;
    //    calculate_all_ratings(&*conn).unwrap();
    //    assert!((*ENTRY_RATINGS.lock().unwrap()).len() > 9_000);
    //    let query = super::SearchQuery {
    //        bbox: "-10,-10,10,10".into(),
    //        categories: None,
    //        text: None,
    //        tags: None,
    //    };
    //    b.iter(move || {
    //        let conn = pool.get().unwrap();
    //        let db = DbConn(conn);
    //        super::get_search(db, query.clone()).unwrap()
    //    });
    //}
}
