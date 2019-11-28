use super::{super::guards::Bearer, geocoding::*, *};

use crate::core::util::{geo::MapBbox, validate};

use rocket::{
    http::{RawStr, Status as HttpStatus},
    request::{FromQuery, Query},
};

fn check_and_set_address_location(e: &mut usecases::NewEvent) {
    // TODO: Parse logical parts of NewEvent earlier
    let addr = Address {
        street: e.street.clone(),
        zip: e.zip.clone(),
        city: e.city.clone(),
        country: e.country.clone(),
    };
    if let Some((lat, lng)) = resolve_address_lat_lng(&addr) {
        e.lat = Some(lat);
        e.lng = Some(lng);
    }
}

#[post("/events", format = "application/json", data = "<e>")]
pub fn post_event_with_token(
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    token: Bearer,
    e: Json<usecases::NewEvent>,
) -> Result<String> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    let event =
        usecases::create_new_event(&*db.exclusive()?, &mut search_engine, Some(&token.0), e)?;
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
//     let id = usecases::create_new_event(&*db, &search_engine, e.clone())?;
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
    _e: Json<usecases::UpdateEvent>,
) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[put("/events/<id>", format = "application/json", data = "<e>")]
pub fn put_event_with_token(
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    token: Bearer,
    id: &RawStr,
    e: Json<usecases::UpdateEvent>,
) -> Result<()> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    usecases::update_event(
        &*db.exclusive()?,
        &mut search_engine,
        Some(&token.0),
        &id.to_string(),
        e,
    )?;
    Ok(Json(()))
}

impl<'q> FromQuery<'q> for usecases::EventQuery {
    type Error = crate::core::prelude::Error;

    fn from_query(query: Query<'q>) -> std::result::Result<Self, Self::Error> {
        let mut q = usecases::EventQuery::default();

        let tags: Vec<_> = query
            .clone()
            .filter(|i| i.key == "tag")
            .map(|i| i.value.to_string())
            .filter(|v| !v.is_empty())
            .collect();

        if !tags.is_empty() {
            q.tags = Some(tags);
        }

        q.created_by = query
            .clone()
            .filter(|i| i.key == "created_by")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0)
            .map(|s| s.parse())
            .transpose()
            .map_err(|_| ParameterError::Email)?;

        let start_min = query
            .clone()
            .filter(|i| i.key == "start_min")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(s) = start_min {
            let x: i64 = s.parse()?;
            q.start_min = Some(Timestamp::from_inner(x));
        }

        let start_max = query
            .clone()
            .filter(|i| i.key == "start_max")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(e) = start_max {
            let x: i64 = e.parse()?;
            q.start_max = Some(Timestamp::from_inner(x));
        }

        let bbox = query
            .clone()
            .filter(|i| i.key == "bbox")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(bbox) = bbox {
            let bbox = bbox
                .parse::<MapBbox>()
                .map_err(|_err| ParameterError::Bbox)?;
            validate::bbox(&bbox)?;
            q.bbox = Some(bbox);
        }

        q.text = query
            .filter(|i| i.key == "text")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);

        Ok(q)
    }
}

#[get("/events?<query..>")]
pub fn get_events_with_token(
    db: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    token: Bearer,
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    //TODO: check token
    let events = usecases::query_events(&*db.shared()?, &search_engine, query, Some(token.0))?
        .into_iter()
        .map(json::Event::from)
        .collect();
    Ok(Json(events))
}

#[get("/events?<query..>", rank = 2)]
pub fn get_events_chronologically(
    db: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }
    let events: Vec<_> = usecases::query_events(&*db.shared()?, &search_engine, query, None)?
        .into_iter()
        .map(|e| {
            // TODO:Make sure within usecase that the creator email
            // is not shown to unregistered users
            Event {
                created_by: None,
                ..e
            }
        })
        .map(json::Event::from)
        .collect();
    Ok(Json(events))
}

#[delete("/events/<_id>", rank = 2)]
pub fn delete_event(mut _db: sqlite::Connections, _id: &RawStr) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[delete("/events/<id>")]
pub fn delete_event_with_token(db: sqlite::Connections, token: Bearer, id: &RawStr) -> Result<()> {
    usecases::delete_event(&mut *db.exclusive()?, &token.0, &id.to_string())?;
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use super::*;
    use rocket::http::Header;

    mod create {
        use super::*;

        #[test]
        fn without_creator_email() {
            let (client, _db) = setup();
            let req = client
                .post("/events")
                .header(ContentType::JSON)
                .body(r#"{"title":"foo","start":1234}"#);
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
                .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#);
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
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Forbidden);
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
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
                    .body(r#"{"title":"Reginaltreffen","start":0,"created_by":"a-very-super-long-email-address@a-super-long-domain.com"}"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","email":"","homepage":"","description":"","registration":""}"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"telephone","telephone":"12345"}"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","tags":["a"] }"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Ok);
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer a"))
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","tags":["b"] }"#)
                    .dispatch();
                assert_eq!(res.status(), HttpStatus::Forbidden);
                let res = client
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer b"))
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","tags":["b"] }"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","tags":["", " "," tag","tag ","two tags", "tag"]}"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"foo"}"#)
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
                    .body(r#"{"title":"x","start":0}"#)
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
                    .body(r#"{"title":"","start":0,"created_by":"foo@bar.com"}"#)
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"telephone"}"#)
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
                .body(r#"{"title":"x","start":0}"#)
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
            let e = usecases::NewEvent {
                title: "x".into(),
                start: 0,
                tags: Some(vec!["bla".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            };
            let e =
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
            let req = client
                .get(format!("/events/{}", e.id))
                .header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), HttpStatus::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(
                body_str,
                format!("{{\"id\":\"{}\",\"title\":\"x\",\"start\":0,\"email\":\"test@example.com\",\"tags\":[\"bla\"],\"registration\":\"email\"}}", e.id)
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
                        start: NaiveDateTime::from_timestamp(0, 0),
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
            let event_start_times = vec![100, 0, 300, 50, 200];
            for start in event_start_times {
                let e = usecases::NewEvent {
                    title: start.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
            }
            let mut res = client.get("/events").header(ContentType::JSON).dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert!(objects[0].contains("\"start\":0"));
            assert!(objects[1].contains("\"start\":50"));
            assert!(objects[2].contains("\"start\":100"));
            assert!(objects[3].contains("\"start\":200"));
            assert!(objects[4].contains("\"start\":300"));
        }

        #[test]
        fn filtered_by_tags() {
            let (client, db, mut search_engine) = setup2();
            let tags = vec![vec!["a"], vec!["b"], vec!["c"], vec!["a", "b"]];
            for tags in tags {
                let e = usecases::NewEvent {
                    title: format!("{:?}", tags),
                    start: 0,
                    tags: Some(tags.into_iter().map(str::to_string).collect()),
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
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
                        ..Default::default()
                    };
                    usecases::create_new_event(
                        &*db.exclusive().unwrap(),
                        &mut search_engine,
                        Some("foo"),
                        new_event,
                    )
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
            let event_start_times = vec![100, 0, 300, 50, 200];
            for start in event_start_times {
                let e = usecases::NewEvent {
                    title: start.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
            }
            let mut res = client
                .get("/events?start_min=150")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 2);
            assert!(objects[0].contains("\"title\":\"200\""));
            assert!(objects[1].contains("\"title\":\"300\""));
        }

        #[test]
        fn filtered_by_start_max() {
            let (client, db, mut search_engine) = setup2();
            let event_start_times = vec![100, 0, 300, 50, 200];
            for start in event_start_times {
                let e = usecases::NewEvent {
                    title: start.to_string(),
                    start,
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
            }
            let mut res = client
                .get("/events?start_max=250")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 4);
            assert!(objects[0].contains("\"title\":\"0\""));
            assert!(objects[1].contains("\"title\":\"50\""));
            assert!(objects[2].contains("\"title\":\"100\""));
            assert!(objects[3].contains("\"title\":\"200\""));
        }

        #[test]
        fn filtered_by_bounding_box() {
            let (client, db, mut search_engine) = setup2();
            let coordinates = &[(-8.0, 0.0), (0.3, 5.0), (7.0, 7.9), (12.0, 0.0)];
            for &(lat, lng) in coordinates {
                let e = usecases::NewEvent {
                    title: format!("{}-{}", lat, lng),
                    start: 0,
                    lat: Some(lat),
                    lng: Some(lng),
                    created_by: Some("test@example.com".into()),
                    ..Default::default()
                };
                usecases::create_new_event(&*db.exclusive().unwrap(), &mut search_engine, None, e)
                    .unwrap();
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

        #[test]
        fn without_api_token() {
            let (client, _) = setup();
            let res = client
                .put("/events/foo")
                .header(ContentType::JSON)
                .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
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
                .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
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
                ..Default::default()
            };
            let id = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e,
            )
            .unwrap()
            .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":5,"created_by":"changed@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "new");
            assert_eq!(new.start.timestamp(), 5);
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
                ..Default::default()
            };
            let id = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("bar"),
                e,
            )
            .unwrap()
            .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":5,"created_by":"changed@bar.com"}"#)
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
                ..Default::default()
            };
            let id = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e,
            )
            .unwrap()
            .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":5,"created_by":"changed@bar.com","tags":["bla2"]}"#)
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
                ..Default::default()
            };
            let id = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e,
            )
            .unwrap()
            .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":5,"created_by":"changed@bar.com","tags":["blub","new","org-tag2"]}"#)
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.exclusive().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.tags, vec!["blub", "new", "org-tag2"]);
        }

        #[test]
        fn with_api_token_without_created_by() {
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
            let e = usecases::NewEvent {
                title: "x".into(),
                tags: Some(vec!["bla".into()]),
                created_by: created_by.clone(),
                ..Default::default()
            };
            let id = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e,
            )
            .unwrap()
            .id;
            assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
            let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body("{\"title\":\"Changed\",\"start\":99}")
                .dispatch();
            assert_eq!(res.status(), HttpStatus::Ok);
            let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
            assert_eq!(new.title, "Changed");
            assert_eq!(new.created_by, created_by);
        }
    }

    mod delete {
        use super::*;

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
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id1 = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e1,
            )
            .unwrap()
            .id;
            let e2 = usecases::NewEvent {
                title: "x".into(),
                start: 0,
                tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
                created_by: Some("foo@bar.com".into()),
                ..Default::default()
            };
            let id2 = usecases::create_new_event(
                &*db.exclusive().unwrap(),
                &mut search_engine,
                Some("foo"),
                e2,
            )
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
    }
}
