use super::{super::guards::Bearer, geocoding::*, *};

use crate::{
    adapters,
    core::{
        prelude::Result as CoreResult,
        util::{geo::MapBbox, validate},
    },
    infrastructure::flows::prelude as flows,
};

use rocket::{
    http::{RawStr, Status as HttpStatus},
    request::{FromQuery, Query},
};

fn check_and_set_address_location(e: &mut usecases::NewEvent) {
    if e.lat.is_some() && e.lng.is_some() {
        // Preserve given locations
        return;
    }
    // TODO: Parse logical parts of NewEvent earlier
    let addr = Address {
        street: e.street.clone(),
        zip: e.zip.clone(),
        city: e.city.clone(),
        country: e.country.clone(),
        state: e.state.clone(),
    };
    if let Some((lat, lng)) = resolve_address_lat_lng(&addr) {
        e.lat = Some(lat);
        e.lng = Some(lng);
    }
}

#[post("/events", format = "application/json", data = "<e>")]
pub fn post_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    token: Bearer,
    e: Json<usecases::NewEvent>,
) -> Result<String> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    let event = flows::create_event(&connections, &mut search_engine, Some(&token.0), e)?;
    Ok(Json(event.id.to_string()))
}

#[post("/events", format = "application/json", data = "<_e>", rank = 2)]
// NOTE:
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn post_event(mut _db: sqlite::Connections, _e: Json<usecases::NewEvent>) -> HttpStatus {
    HttpStatus::Unauthorized
}
// But in the future we might allow anonymous event creation:
//
// pub fn post_event(mut db: sqlite::Connections, e: Json<usecases::NewEvent>) -> Result<String> {
//     let mut e = e.into_inner();
//     e.created_by = None; // ignore because of missing authorization
//     e.token = None; // ignore token
//     let id = flows::create_event(&*db, &search_engine, e.clone())?;
//     Ok(Json(id))
// }

#[get("/events/<id>")]
pub fn get_event(db: sqlite::Connections, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&*db.shared()?, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[put("/events/<_id>", format = "application/json", data = "<_e>", rank = 2)]
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn put_event(
    mut _db: sqlite::Connections,
    _id: &RawStr,
    _e: Json<usecases::NewEvent>,
) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[put("/events/<id>", format = "application/json", data = "<e>")]
pub fn put_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    token: Bearer,
    id: &RawStr,
    e: Json<usecases::NewEvent>,
) -> Result<()> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    flows::update_event(
        &connections,
        &mut search_engine,
        Some(&token.0),
        id.to_string().into(),
        e,
    )?;
    Ok(Json(()))
}

impl<'q> FromQuery<'q> for usecases::EventQuery {
    type Error = crate::core::prelude::Error;

    fn from_query(query: Query<'q>) -> std::result::Result<Self, Self::Error> {
        let created_by = query
            .clone()
            .filter(|i| i.key == "created_by")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
            .map(|s| s.parse::<Email>())
            .transpose()
            .map_err(|_| ParameterError::Email)?;

        let bbox = if let Some(bbox) = query
            .clone()
            .filter(|i| i.key == "bbox")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            let bbox = bbox
                .parse::<MapBbox>()
                .map_err(|_err| ParameterError::Bbox)?;
            validate::bbox(&bbox)?;
            Some(bbox)
        } else {
            None
        };

        let limit = if let Some(limit) = query
            .clone()
            .filter(|i| i.key == "limit")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(validate_and_adjust_query_limit(limit.parse()?)?)
        } else {
            None
        };

        let start_max = if let Some(start_max) = query
            .clone()
            .filter(|i| i.key == "start_max")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(Timestamp::from_inner(start_max.parse()?))
        } else {
            None
        };

        let start_min = if let Some(start_min) = query
            .clone()
            .filter(|i| i.key == "start_min")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(Timestamp::from_inner(start_min.parse()?))
        } else {
            None
        };

        let tags: Vec<_> = query
            .clone()
            .filter(|i| i.key == "tag")
            .map(|i| i.value.to_string())
            .filter(|v| !v.is_empty())
            .collect();
        let tags = if tags.is_empty() { None } else { Some(tags) };

        let text = query
            .clone()
            .filter(|i| i.key == "text")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty());

        drop(query); // silence clippy warning
        Ok(usecases::EventQuery {
            bbox,
            created_by,
            limit,
            start_max,
            start_min,
            tags,
            text,
        })
    }
}

const MAX_RESULT_LIMIT: usize = 500;

#[allow(clippy::absurd_extreme_comparisons)]
fn validate_and_adjust_query_limit(limit: usize) -> CoreResult<usize> {
    if limit > MAX_RESULT_LIMIT {
        info!(
            "Requested limit {} exceeds maximum limit {} for event search results",
            limit, MAX_RESULT_LIMIT
        );
        Ok(MAX_RESULT_LIMIT)
    } else if limit <= 0 {
        warn!("Invalid search limit: {}", limit);
        Err(Error::Parameter(ParameterError::InvalidLimit))
    } else {
        Ok(limit)
    }
}

#[get("/events?<query..>")]
pub fn get_events_with_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    token: Bearer,
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    let db = connections.shared()?;
    let org = usecases::authorize_organization_by_token(&*db, &token.0)?;
    let events = usecases::query_events(&*db, &search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let owned_tags = org.owned_tags;
    let events: Vec<_> = events
        .into_iter()
        .map(|e| usecases::filter_event(e, owned_tags.iter().map(String::as_str)))
        .map(json::Event::from)
        .collect();

    Ok(Json(events))
}

#[get("/events?<query..>", rank = 2)]
pub fn get_events_chronologically(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }

    let db = connections.shared()?;
    let events = usecases::query_events(&*db, &search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let owned_tags = vec![];
    let events: Vec<_> = events
        .into_iter()
        .map(|e| usecases::filter_event(e, owned_tags.iter().map(String::as_str)))
        .map(json::Event::from)
        .collect();

    Ok(Json(events))
}

#[get("/export/events.csv?<query..>")]
pub fn csv_export_with_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    token: Bearer,
    login: Login,
    query: usecases::EventQuery,
) -> result::Result<Content<String>, AppError> {
    let organization =
        usecases::authorize_organization_by_token(&*connections.shared()?, &token.0)?;
    csv_export(connections, search_engine, Some(organization), login, query)
}

#[get("/export/events.csv?<query..>", rank = 2)]
pub fn csv_export_without_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    login: Login,
    query: usecases::EventQuery,
) -> result::Result<Content<String>, AppError> {
    csv_export(connections, search_engine, None, login, query)
}

fn csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    org: Option<Organization>,
    login: Login,
    query: usecases::EventQuery,
) -> result::Result<Content<String>, AppError> {
    let owned_tags = org.map(|org| org.owned_tags).unwrap_or_default();

    let db = connections.shared()?;
    let user = usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?;
    let limit = if let Some(limit) = query.limit {
        // Limited
        limit
    } else {
        // Unlimited
        db.count_events()? + 100
    };
    let query = usecases::EventQuery {
        limit: Some(limit),
        ..query
    };
    let events = usecases::query_events(&*db, &search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let events = events
        .into_iter()
        .map(|e| usecases::export_event(e, user.role, owned_tags.iter().map(String::as_str)));

    let records: Vec<_> = events.map(adapters::csv::EventRecord::from).collect();

    let buff: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buff);

    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(Content(ContentType::CSV, data))
}

#[post("/events/<ids>/archive")]
pub fn post_events_archive(
    login: Login,
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    ids: String,
) -> StatusResult {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyIdList).into());
    }
    let archived_by_email = {
        let db = db.shared()?;
        // Only scouts and admins are entitled to review events
        usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?.email
    };
    let update_count = flows::archive_events(&db, &mut search_engine, &ids, &archived_by_email)?;
    if update_count < ids.len() {
        log::info!(
            "Archived only {} of {} event(s): {:?}",
            update_count,
            ids.len(),
            ids
        );
    }
    Ok(HttpStatus::NoContent)
}

#[delete("/events/<_id>", rank = 2)]
pub fn delete_event(mut _db: sqlite::Connections, _id: &RawStr) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[delete("/events/<id>")]
pub fn delete_event_with_token(
    db: sqlite::Connections,
    token: Bearer,
    id: &RawStr,
) -> StatusResult {
    usecases::delete_event(&mut *db.exclusive()?, &token.0, &id.to_string())?;
    // TODO: Replace with HttpStatus::NoContent
    Ok(HttpStatus::Ok)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use super::*;
    use rocket::http::Header;

    mod create {
        use super::*;

        // Magic time constant: 4132508400 ~= 2030-01-01 00:00:00 + 70 years

        #[test]
        fn without_creator_email() {
            let (client, _db) = setup();
            let req = client
                .post("/events")
                .header(ContentType::JSON)
                .body(r#"{"title":"foo","start":4132508400}"#);
            let response = req.dispatch();

            // NOTE:
            // At the moment we don't want to allow anonymous event creation.
            // So for now we assure that it's blocked:
            assert_eq!(response.status(), HttpStatus::Unauthorized);
            // But in the future we might allow anonymous event creation:
            //
            // assert_eq!(response.status(), HttpStatus::Ok);
            // test_json(&response);
            // let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            // let eid = db.get().unwrap().all_events_chronologically().unwrap()[0].id.clone();
            // assert_eq!(body_str, format!("\"{}\"", eid));
        }

        #[test]
        fn without_api_token_but_with_creator_email() {
            let (client, _db) = setup();
            let req = client
                .post("/events")
                .header(ContentType::JSON)
                .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#);
            let response = req.dispatch();
            // NOTE:
            // At the moment we don't want to allow anonymous event creation.
            // So for now we assure that it's blocked:
            assert_eq!(response.status(), HttpStatus::Unauthorized);
            // But in the future we might allow anonymous event creation:
            //
            // assert_eq!(response.status(), HttpStatus::Ok);
            // test_json(&response);
            // let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            // let ev = db.get().unwrap().all_events_chronologically().unwrap()[0].clone();
            // let eid = ev.id.clone();
            // assert!(ev.created_by.is_none());
            // assert_eq!(body_str, format!("\"{}\"", eid));
            // let req = client
            //     .get(format!("/events/{}", eid))
            //     .header(ContentType::JSON);
            // let mut response = req.dispatch();
            // assert_eq!(response.status(), HttpStatus::Ok);
            // test_json(&response);
            // let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            // assert_eq!(
            //     body_str,
            //     format!(
            //         "{{\"id\":\"{}\",\"title\":\"x\",\"start\":0,\"lat\":0.0,\"lng\":0.0,\"tags\":[]}}",
            //         eid
            //     )
            // );
        }

        mod with_api_token {
            use super::*;

            #[test]
            fn for_organization_without_any_owned_tags() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec![],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let mut res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                test_json(&res);
                let body_str = res.body().and_then(|b| b.into_string()).unwrap();
                let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
                let eid = ev.id.clone();
                assert_eq!(ev.created_by.unwrap(), "foo@bar.com");
                assert_eq!(body_str, format!("\"{}\"", eid));
            }

            #[test]
            fn with_creator_email() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let mut res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                test_json(&res);
                let body_str = res.body().and_then(|b| b.into_string()).unwrap();
                let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
                let eid = ev.id.clone();
                assert_eq!(ev.created_by.unwrap(), "foo@bar.com");
                assert_eq!(body_str, format!("\"{}\"", eid));
            }

            #[test]
            fn with_a_very_long_email() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"Reginaltreffen","start":4132508400,"created_by":"a-very-super-long-email-address@a-super-long-domain.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                let u = db.shared().unwrap().all_users().unwrap()[0].clone();
                assert_eq!(
                    u.email,
                    "a-very-super-long-email-address@a-super-long-domain.com"
                );
            }

            #[test]
            fn with_empty_strings_for_optional_fields() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","email":"","homepage":"","description":"","registration":""}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                test_json(&res);
                let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
                assert!(ev.contact.is_none());
                assert!(ev.homepage.is_none());
                assert!(ev.description.is_none());
            }

            #[test]
            fn with_registration_type() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"telephone","telephone":"12345"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                test_json(&res);
                let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
                assert_eq!(ev.registration.unwrap(), RegistrationType::Phone);
            }

            #[test]
            fn with_reseved_tag_from_foreign_org() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "a".into(),
                        name: "a".into(),
                        owned_tags: vec!["a".into()],
                        api_token: "a".into(),
                    })
                    .unwrap();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "b".into(),
                        name: "b".into(),
                        owned_tags: vec!["b".into()],
                        api_token: "b".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer a"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["a"] }"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer a"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["b"] }"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Forbidden);
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer b"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["b"] }"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
            }

            #[test]
            fn with_spaces_in_tags() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["", " "," tag","tag ","two tags", "tag"]}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                test_json(&res);
                let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
                assert_eq!(
                    ev.tags,
                    // Including the implicitly added org tag
                    vec![
                        "tag".to_string(),
                        "tags".to_string(),
                        "two".to_string(),
                        "org-tag".to_string()
                    ]
                );
            }

            #[test]
            fn with_invalid_registration_type() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"foo"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::BadRequest);
            }

            #[test]
            fn without_creator_email() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::BadRequest);
            }

            #[test]
            fn with_empty_title() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"","start":4132508400,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::BadRequest);
            }

            #[test]
            fn with_phone_registration_but_without_phone_nr() {
                let (client, db) = setup();
                db.exclusive()
                    .unwrap()
                    .create_org(Organization {
                        id: "foo".into(),
                        name: "bar".into(),
                        owned_tags: vec!["org-tag".into()],
                        api_token: "foo".into(),
                    })
                    .unwrap();
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"telephone"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::BadRequest);
            }
        }

        #[test]
        fn with_invalid_api_token() {
            let (client, _) = setup();
            let res = client
                .post("/events")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer not-valid"))
                .body(r#"{"title":"x","start":4132508400}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }
    }

    mod read {
        use super::*;
        use chrono::prelude::*;

        #[test]
        fn by_id() {
            let (client, db, mut search_engine) = setup2();
            let now = Utc::now().naive_utc().timestamp();
            let e = usecases::NewEvent {
                title: "x".into(),
                start: now,
                tags: Some(vec!["bla".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            };
            let e = flows::create_event(&db, &mut search_engine, None, e).unwrap();
            let req = client
                .get(format!("/events/{}", e.id))
                .header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(
                body_str,
                format!("{{\"id\":\"{}\",\"title\":\"x\",\"start\":{},\"email\":\"test@example.com\",\"tags\":[\"bla\"],\"registration\":\"email\"}}", e.id, now)
            );
        }

        #[test]
        fn all() {
            let (client, db) = setup();
            let event_ids = vec!["a", "b", "c"];
            for id in event_ids {
                db.exclusive()
                    .unwrap()
                    .create_event(Event {
                        id: id.into(),
                        title: id.into(),
                        description: None,
                        start: Utc::now().naive_utc(),
                        end: None,
                        location: None,
                        contact: None,
                        tags: vec![],
                        homepage: None,
                        created_by: None,
                        registration: None,
                        organizer: None,
                        archived: None,
                        image_url: None,
                        image_link_url: None,
                    })
                    .unwrap();
            }
            let req = client.get("/events").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"id\":\"a\""));
        }

        #[test]
        fn sorted_by_start() {
            let (client, db, mut search_engine) = setup2();
            let now = Utc::now().naive_utc().timestamp();
            let start_offsets = vec![100, 0, 300, 50, 200];
            for start_offset in start_offsets {
                let start = now + start_offset;
                let e = usecases::NewEvent {
                    title: start_offset.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                flows::create_event(&db, &mut search_engine, None, e).unwrap();
            }
            let mut res = client.get("/events").header(ContentType::JSON).dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert!(objects[0].contains(&format!("\"start\":{}", now)));
            assert!(objects[1].contains(&format!("\"start\":{}", now + 50)));
            assert!(objects[2].contains(&format!("\"start\":{}", now + 100)));
            assert!(objects[3].contains(&format!("\"start\":{}", now + 200)));
            assert!(objects[4].contains(&format!("\"start\":{}", now + 300)));
        }

        #[test]
        fn filtered_by_tags() {
            let (client, db, mut search_engine) = setup2();
            let tags = vec![vec!["a"], vec!["b"], vec!["c"], vec!["a", "b"]];
            for tags in tags {
                let e = usecases::NewEvent {
                    title: format!("{:?}", tags),
                    start: Utc::now().naive_utc().timestamp(),
                    tags: Some(tags.into_iter().map(str::to_string).collect()),
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                flows::create_event(&db, &mut search_engine, None, e).unwrap();
            }

            let req = client.get("/events?tag=a").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"tags\":[\"a\"]"));
            assert!(!body_str.contains("\"tags\":[\"b\"]"));
            assert!(!body_str.contains("\"tags\":[\"c\"]"));
            assert!(body_str.contains("\"tags\":[\"a\",\"b\"]"));

            let req = client.get("/events?tag=b").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("\"tags\":[\"a\"]"));
            assert!(body_str.contains("\"tags\":[\"b\"]"));
            assert!(!body_str.contains("\"tags\":[\"c\"]"));
            assert!(body_str.contains("\"tags\":[\"a\",\"b\"]"));

            let req = client.get("/events?tag=c").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("\"tags\":[\"a\"]"));
            assert!(!body_str.contains("\"tags\":[\"b\"]"));
            assert!(body_str.contains("\"tags\":[\"c\"]"));
            assert!(!body_str.contains("\"tags\":[\"a\",\"b\"]"));

            let req = client.get("/events?tag=a&tag=b").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("\"tags\":[\"a\"]"));
            assert!(!body_str.contains("\"tags\":[\"b\"]"));
            assert!(!body_str.contains("\"tags\":[\"c\"]"));
            assert!(body_str.contains("\"tags\":[\"a\",\"b\"]"));
        }

        #[test]
        fn filtered_by_creator_without_api_token() {
            let (client, _db) = setup();
            let res = client
                .get("/events?created_by=foo%40bar.com")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn filtered_by_creator_with_valid_api_token() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let ids: Vec<_> = ["foo@bar.com", "test@test.com", "bla@bla.bla"]
                .iter()
                .map(|m| {
                    let m = *m;
                    let new_event = usecases::NewEvent {
                        title: m.to_string(),
                        created_by: Some(m.to_string()),
                        start: Utc::now().naive_utc().timestamp(),
                        ..Default::default()
                    };
                    flows::create_event(&db, &mut search_engine, Some("foo"), new_event)
                        .unwrap()
                        .id
                })
                .collect();
            let mut res = client
                .get("/events?created_by=test%40test.com")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains(&format!("\"id\":\"{}\"", ids[0])));
            assert!(body_str.contains(&format!("\"id\":\"{}\"", ids[1])));
            assert!(!body_str.contains(&format!("\"id\":\"{}\"", ids[2])));
        }

        #[test]
        fn filtered_by_creator_with_invalid_api_token() {
            let (client, db) = setup();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();

            let res = client
                .get("/events?created_by=foo@bar.com")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn filtered_by_start_min() {
            let (client, db, mut search_engine) = setup2();
            let now = Utc::now().naive_utc().timestamp();
            let start_offsets = vec![100, 0, 300, 50, 200];
            for start_offset in start_offsets {
                let start = now + start_offset;
                let e = usecases::NewEvent {
                    title: start_offset.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                flows::create_event(&db, &mut search_engine, None, e).unwrap();
            }
            let mut res = client
                .get(format!("/events?start_min={}", now + 150))
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 2);
            assert!(objects[0].contains(&format!("\"start\":{}", now + 200)));
            assert!(objects[1].contains(&format!("\"start\":{}", now + 300)));
        }

        #[test]
        fn filtered_by_start_max() {
            let (client, db, mut search_engine) = setup2();
            let now = Utc::now().naive_utc().timestamp();
            let start_offsets = vec![100, 0, 300, 50, 200];
            for start_offset in start_offsets {
                let start = now + start_offset;
                let e = usecases::NewEvent {
                    title: start.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                flows::create_event(&db, &mut search_engine, None, e).unwrap();
            }
            let mut res = client
                .get(format!("/events?start_max={}", now + 250))
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 4);
            assert!(objects[0].contains(&format!("\"start\":{}", now)));
            assert!(objects[1].contains(&format!("\"start\":{}", now + 50)));
            assert!(objects[2].contains(&format!("\"start\":{}", now + 100)));
            assert!(objects[3].contains(&format!("\"start\":{}", now + 200)));
        }

        #[test]
        fn filtered_by_bounding_box() {
            let (client, db, mut search_engine) = setup2();
            let coordinates = &[(-8.0, 0.0), (0.3, 5.0), (7.0, 7.9), (12.0, 0.0)];
            for &(lat, lng) in coordinates {
                let e = usecases::NewEvent {
                    title: format!("{}-{}", lat, lng),
                    start: Utc::now().naive_utc().timestamp(),
                    lat: Some(lat),
                    lng: Some(lng),
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                flows::create_event(&db, &mut search_engine, None, e).unwrap();
            }
            let mut res = client
                .get("/events?bbox=-8,-5,10,7.9")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"title\":\"-8-0\""));
            assert!(body_str.contains("\"title\":\"7-7.9\""));
            assert!(body_str.contains("\"title\":\"0.3-5\""));
            assert!(!body_str.contains("\"title\":\"12-0\""));

            let mut res = client
                .get("/events?bbox=10,-1,13,1")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("\"title\":\"-8-0\""));
            assert!(!body_str.contains("\"title\":\"7-7.9\""));
            assert!(!body_str.contains("\"title\":\"0.3-5\""));
            assert!(body_str.contains("\"title\":\"12-0\""));
        }
    }

    mod update {
        use super::*;
        use chrono::prelude::*;

        #[test]
        fn without_api_token() {
            let (client, _) = setup();
            let res = client
                .put("/events/foo")
                .header(ContentType::JSON)
                .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn with_invalid_api_token() {
            let (client, db) = setup();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let res = client
                .put("/events/foo")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn with_api_token() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into(), "org-tag".into()]),
                created_by: Some("foo@bar.com".into()),
                start: Utc::now().naive_utc().timestamp(),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "new");
            assert_eq!(new.start.timestamp(), 4_132_508_400);
            assert_eq!(new.created_by.unwrap(), "changed@bar.com");
        }

        #[test]
        fn with_api_token_for_organization_without_any_owned_tags() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into()]),
                created_by: Some("foo@bar.com".into()),
                start: Utc::now().naive_utc().timestamp(),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "new");
            assert_eq!(new.start.timestamp(), 4_132_508_400);
            assert_eq!(new.created_by.unwrap(), "changed@bar.com");
        }

        #[test]
        fn with_api_token_but_mismatching_tag() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            // The events needs an owner, otherwise the test may fail
            // with a debug assertion.
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "bar".into(),
                    name: "foo".into(),
                    owned_tags: vec!["bla".into()],
                    api_token: "bar".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into()]),
                created_by: Some("foo@bar.com".into()),
                start: Utc::now().naive_utc().timestamp(),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("bar"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Forbidden);
        }

        #[test]
        fn with_api_token_keep_org_tag() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into(), "org-tag".into()]),
                created_by: Some("foo@bar.com".into()),
                start: Utc::now().naive_utc().timestamp(),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com","tags":["bla2"]}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.exclusive().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.tags, vec!["bla2", "org-tag"]);
        }

        #[test]
        fn with_api_token_and_removing_tag() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag1".into(), "org-tag2".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec![
                    "bli".into(),
                    "org-tag".into(),
                    "org-tag1".into(),
                    "bla".into(),
                    "blub".into(),
                ]),
                created_by: Some("foo@bar.com".into()),
                start: Utc::now().naive_utc().timestamp(),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com","tags":["blub","new","org-tag2"]}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.exclusive().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.tags, vec!["blub", "new", "org-tag2"]);
        }

        #[test]
        fn with_api_token_created_by() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["bla".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let created_by = Some("foo@bar.com".into());
            let start = Utc::now().naive_utc().timestamp();
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into()]),
                created_by: created_by.clone(),
                start,
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());

            // Without created_by
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body("{\"title\":\"Changed\",\"start\":4132508400}")
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "Changed");
            // created_by is unmodified
            assert_eq!(new.created_by, created_by);

            // With created_by
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(&format!("{{\"title\":\"Changed again\",\"created_by\":\"changed@bar.com\",\"start\":{}}}", start))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "Changed again");
            // created_by has been updated
            assert_eq!(new.created_by, Some("changed@bar.com".into()));
        }
    }

    mod archive {
        use super::*;
        use chrono::prelude::*;

        #[test]
        fn only_scouts_and_admins_can_archive_events() {
            let (client, db) = setup();

            let users = vec![
                User {
                    email: "admin@example.com".into(),
                    email_confirmed: true,
                    password: "secret".parse::<Password>().unwrap(),
                    role: Role::Admin,
                },
                User {
                    email: "scout@example.com".into(),
                    email_confirmed: true,
                    password: "secret".parse::<Password>().unwrap(),
                    role: Role::Scout,
                },
                User {
                    email: "user@example.com".into(),
                    email_confirmed: true,
                    password: "secret".parse::<Password>().unwrap(),
                    role: Role::User,
                },
            ];
            for u in users {
                db.exclusive().unwrap().create_user(&u).unwrap();
            }

            let response = client.post("/events/foo/archive").dispatch();
            assert_eq!(response.status(), Status::Unauthorized);

            let login = client
                .post("/login")
                .header(ContentType::JSON)
                .body(r#"{"email": "user@example.com", "password": "secret"}"#)
                .dispatch();
            assert_eq!(login.status(), Status::Ok);
            let response = client.post("/events/foo/archive").dispatch();
            assert_eq!(response.status(), Status::Unauthorized);

            let login = client
                .post("/login")
                .header(ContentType::JSON)
                .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
                .dispatch();
            assert_eq!(login.status(), Status::Ok);
            let response = client.post("/events/foo/archive").dispatch();
            assert_eq!(response.status(), Status::NoContent);

            let login = client
                .post("/login")
                .header(ContentType::JSON)
                .body(r#"{"email": "admin@example.com", "password": "secret"}"#)
                .dispatch();
            assert_eq!(login.status(), Status::Ok);
            let response = client.post("/events/foo/archive").dispatch();
            assert_eq!(response.status(), Status::NoContent);
        }

        #[test]
        fn archive_events() {
            let (client, db, mut search_engine) = setup2();

            let admin = User {
                email: "admin@example.com".into(),
                email_confirmed: true,
                password: "secret".parse::<Password>().unwrap(),
                role: Role::Admin,
            };
            db.exclusive().unwrap().create_user(&admin).unwrap();

            // Create 2 events
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e1 = usecases::NewEvent {
                title: "x".into(),
                start: Utc::now().naive_utc().timestamp(),
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id1 = flows::create_event(&db, &mut search_engine, Some("foo"), e1)
                .unwrap()
                .id;
            let e2 = usecases::NewEvent {
                title: "x".into(),
                start: Utc::now().naive_utc().timestamp(),
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id2 = flows::create_event(&db, &mut search_engine, Some("foo"), e2)
                .unwrap()
                .id;

            let mut response = client.get("/events").dispatch();
            assert_eq!(response.status(), Status::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains(&format!("\"id\":\"{}\"", id1)));
            assert!(body_str.contains(&format!("\"id\":\"{}\"", id2)));

            let login = client
                .post("/login")
                .header(ContentType::JSON)
                .body(r#"{"email": "admin@example.com", "password": "secret"}"#)
                .dispatch();
            assert_eq!(login.status(), Status::Ok);

            let response = client.post(format!("/events/{}/archive", id2)).dispatch();
            assert_eq!(response.status(), Status::NoContent);

            let mut response = client.get("/events").dispatch();
            assert_eq!(response.status(), Status::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains(&format!("\"id\":\"{}\"", id1)));
            assert!(!body_str.contains(&format!("\"id\":\"{}\"", id2)));

            let response = client
                .post(format!("/events/{},{}/archive", id1, id2))
                .dispatch();
            assert_eq!(response.status(), Status::NoContent);

            let mut response = client.get("/events").dispatch();
            assert_eq!(response.status(), Status::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains(&format!("\"id\":\"{}\"", id1)));
            assert!(!body_str.contains(&format!("\"id\":\"{}\"", id2)));
        }
    }

    mod delete {
        use super::*;
        use chrono::prelude::*;

        #[test]
        fn without_api_token() {
            let (client, _) = setup();
            let res = client
                .delete("/events/foo")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn with_invalid_api_token() {
            let (client, db) = setup();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["org-tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let res = client
                .delete("/events/foo")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }

        #[test]
        fn with_api_token() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec!["tag".into()],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e1 = usecases::NewEvent {
                title: "x".into(),
                start: Utc::now().naive_utc().timestamp(),
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id1 = flows::create_event(&db, &mut search_engine, Some("foo"), e1)
                .unwrap()
                .id;
            let e2 = usecases::NewEvent {
                title: "x".into(),
                start: Utc::now().naive_utc().timestamp(),
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id2 = flows::create_event(&db, &mut search_engine, Some("foo"), e2)
                .unwrap()
                .id;
            // Manually delete the implicitly added org tag from the 2nd event!
            let mut e2 = db.shared().unwrap().get_event(id2.as_ref()).unwrap();
            e2.tags.retain(|t| t != "tag");
            db.exclusive().unwrap().update_event(&e2).unwrap();
            assert_eq!(db.shared().unwrap().count_events().unwrap(), 2);
            // The 1st event has the owned tag and should be deleted.
            let res = client
                .delete(format!("/events/{}", id1))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            // The 2nd event is not tagged with one of the owned tags.
            let res = client
                .delete(format!("/events/{}", id2))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Forbidden);
            assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
        }

        // FIXME: This test should fail, but it doesn't!!
        #[test]
        #[ignore]
        fn with_api_token_by_organization_without_any_owned_tags() {
            let (client, db, mut search_engine) = setup2();
            db.exclusive()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = usecases::NewEvent {
                title: "x".into(),
                start: Utc::now().naive_utc().timestamp(),
                tags: Some(vec!["bla".into()]),
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id = flows::create_event(&db, &mut search_engine, Some("foo"), e)
                .unwrap()
                .id;
            assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
            let res = client
                .delete(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
            assert_eq!(res.status(), HttpStatus::Unauthorized);
        }
    }

    use chrono::prelude::*;
    #[test]
    fn export_csv() {
        let (client, db, mut search_engine) = setup2();

        let users = vec![
            User {
                email: "admin@example.com".into(),
                email_confirmed: true,
                password: "secret".parse::<Password>().unwrap(),
                role: Role::Admin,
            },
            User {
                email: "scout@example.com".into(),
                email_confirmed: true,
                password: "secret".parse::<Password>().unwrap(),
                role: Role::Scout,
            },
            User {
                email: "user@example.com".into(),
                email_confirmed: true,
                password: "secret".parse::<Password>().unwrap(),
                role: Role::User,
            },
        ];
        for u in users {
            db.exclusive().unwrap().create_user(&u).unwrap();
        }

        db.exclusive()
            .unwrap()
            .create_org(Organization {
                id: "foo".into(),
                name: "foo_name".into(),
                owned_tags: vec!["tag".into()],
                api_token: "foo".into(),
            })
            .unwrap();
        db.exclusive()
            .unwrap()
            .create_org(Organization {
                id: "bar".into(),
                name: "bar_name".into(),
                owned_tags: vec!["tag2".into()],
                api_token: "bar".into(),
            })
            .unwrap();
        let start1 = Utc::now().naive_utc().timestamp();
        let e1 = usecases::NewEvent {
            title: "title1".into(),
            start: start1,
            tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
            created_by: Some("createdby1@example.com".into()),
            email: Some("email1@example.com".into()),
            telephone: Some("phone1".into()),
            state: Some("state".into()),
            ..Default::default()
        };
        let id1 = flows::create_event(&db, &mut search_engine, Some("foo"), e1)
            .unwrap()
            .id;
        let start2 = Utc::now().naive_utc().timestamp();
        let e2 = usecases::NewEvent {
            title: "title2".into(),
            start: start2,
            tags: Some(vec!["bli".into()]), // org tag will be added implicitly!
            created_by: Some("createdby2@example.com".into()),
            email: Some("email2@example.com".into()),
            telephone: Some("phone2".into()),
            ..Default::default()
        };
        let id2 = flows::create_event(&db, &mut search_engine, Some("bar"), e2)
            .unwrap()
            .id;

        let response = client.get("/export/events.csv").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        // Regular users are not allowed to export events
        let login = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email": "user@example.com", "password": "secret"}"#)
            .dispatch();
        assert_eq!(login.status(), Status::Ok);
        let req = client.get("/export/events.csv");
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        // Scout without token sees contact details of all events, but not created_by
        let login = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
            .dispatch();
        assert_eq!(login.status(), Status::Ok);
        let mut response = client.get("/export/events.csv").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        eprintln!("{}", body_str);
        assert!(body_str.starts_with("id,created_by,organizer,title,description,start,end,lat,lng,street,zip,city,country,state,email,phone,homepage,image_url,image_link_url,tags\n"));
        assert!(body_str.contains(&format!(
            "{},,,title1,,{},,,,,,,,state,email1@example.com,phone1,,,,\"bla,tag\"\n",
            id1, start1
        )));
        assert!(body_str.contains(&format!(
            "{},,,title2,,{},,,,,,,,,email2@example.com,phone2,,,,\"bli,tag2\"\n",
            id2, start2
        )));
        assert!(!body_str.contains("createdby1@example.com"));
        assert!(!body_str.contains("createdby2@example.com"));

        // Scout with token sees contact details of all events and created_by for their owned events
        let login = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
            .dispatch();
        assert_eq!(login.status(), Status::Ok);
        let mut response = client
            .get("/export/events.csv")
            .header(Header::new("Authorization", "Bearer foo"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.starts_with("id,created_by,organizer,title,description,start,end,lat,lng,street,zip,city,country,state,email,phone,homepage,image_url,image_link_url,tags\n"));
        assert!(body_str.contains(&format!("{},createdby1@example.com,,title1,,{},,,,,,,,state,email1@example.com,phone1,,,,\"bla,tag\"\n", id1, start1)));
        assert!(body_str.contains(&format!(
            "{},,,title2,,{},,,,,,,,,email2@example.com,phone2,,,,\"bli,tag2\"\n",
            id2, start2
        )));
        assert!(!body_str.contains("createdby2@example.com"));

        // Admin without token sees everything
        let login = client
            .post("/login")
            .header(ContentType::JSON)
            .body(r#"{"email": "admin@example.com", "password": "secret"}"#)
            .dispatch();
        assert_eq!(login.status(), Status::Ok);
        let mut response = client.get("/export/events.csv").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.starts_with("id,created_by,organizer,title,description,start,end,lat,lng,street,zip,city,country,state,email,phone,homepage,image_url,image_link_url,tags\n"));
        assert!(body_str.contains(&format!("{},createdby1@example.com,,title1,,{},,,,,,,,state,email1@example.com,phone1,,,,\"bla,tag\"\n", id1, start1)));
        assert!(body_str.contains(&format!(
            "{},createdby2@example.com,,title2,,{},,,,,,,,,email2@example.com,phone2,,,,\"bli,tag2\"\n",
            id2, start2
        )));
    }
}
