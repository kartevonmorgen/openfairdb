use super::{super::guards::Bearer, *};
use rocket::http::RawStr;
use rocket::http::Status;
use rocket::request::{FromQuery, Query};

#[post("/events", format = "application/json", data = "<e>")]
pub fn post_event_with_token(
    mut db: DbConn,
    token: Bearer,
    e: Json<usecases::NewEvent>,
) -> Result<String> {
    let mut e = e.into_inner();
    e.token = Some(token.0);
    let id = usecases::create_new_event(&mut *db, e.clone())?;
    Ok(Json(id))
}

#[post("/events", format = "application/json", data = "<_e>", rank = 2)]
// NOTE:
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn post_event(mut _db: DbConn, _e: Json<usecases::NewEvent>) -> Status {
    Status::Unauthorized
}
// But in the future we might allow anonymous event creation:
//
// pub fn post_event(mut db: DbConn, e: Json<usecases::NewEvent>) -> Result<String> {
//     let mut e = e.into_inner();
//     e.created_by = None; // ignore because of missing authorization
//     e.token = None; // ignore token
//     let id = usecases::create_new_event(&mut *db, e.clone())?;
//     Ok(Json(id))
// }

#[get("/events/<id>")]
pub fn get_event(db: DbConn, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&*db, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[put("/events/<_id>", format = "application/json", data = "<_e>", rank = 2)]
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn put_event(mut _db: DbConn, _id: &RawStr, _e: Json<usecases::UpdateEvent>) -> Status {
    Status::Unauthorized
}

#[put("/events/<id>", format = "application/json", data = "<e>")]
pub fn put_event_with_token(
    mut db: DbConn,
    token: Bearer,
    id: &RawStr,
    e: Json<usecases::UpdateEvent>,
) -> Result<()> {
    let mut e = e.into_inner();
    e.token = Some(token.0);
    usecases::update_event(&mut *db, &id.to_string(), e.clone())?;
    Ok(Json(()))
}

#[derive(Clone, Default)]
pub struct EventQuery {
    tags: Option<Vec<String>>,
    created_by: Option<String>,
    bbox: Option<Bbox>,
    start_min: Option<u64>,
    start_max: Option<u64>,
}

impl<'q> FromQuery<'q> for EventQuery {
    type Error = crate::core::prelude::Error;

    fn from_query(query: Query<'q>) -> std::result::Result<Self, Self::Error> {
        let mut q = EventQuery::default();

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
            .nth(0);

        let start_min = query
            .clone()
            .filter(|i| i.key == "start_min")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(s) = start_min {
            let x = s.parse()?;
            q.start_min = Some(x);
        }

        let start_max = query
            .clone()
            .filter(|i| i.key == "start_max")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(e) = start_max {
            let x = e.parse()?;
            q.start_max = Some(x);
        }

        let bbox = query
            .filter(|i| i.key == "bbox")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);
        if let Some(bbox) = bbox {
            let bbox = geo::extract_bbox(&bbox)?;
            q.bbox = Some(bbox);
        }

        Ok(q)
    }
}

#[get("/events?<query..>")]
pub fn get_events_with_token(
    db: DbConn,
    token: Bearer,
    query: EventQuery,
) -> Result<Vec<json::Event>> {
    //TODO: check token
    let events = usecases::query_events(
        &*db,
        query.tags,
        query.bbox,
        query.start_min,
        query.start_max,
        query.created_by,
        Some(token.0),
    )?;
    let events = events.into_iter().map(json::Event::from).collect();
    Ok(Json(events))
}

#[get("/events?<query..>", rank = 2)]
pub fn get_events(db: DbConn, query: EventQuery) -> Result<Vec<json::Event>> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }
    let events = usecases::query_events(
        &*db,
        query.tags,
        query.bbox,
        query.start_min,
        query.start_max,
        query.created_by,
        None,
    )?;
    let events = events.into_iter().map(json::Event::from).collect();
    Ok(Json(events))
}

#[delete("/events/<_id>", rank = 2)]
pub fn delete_event(mut _db: DbConn, _id: &RawStr) -> Status {
    Status::Unauthorized
}

#[delete("/events/<id>")]
pub fn delete_event_with_token(mut db: DbConn, token: Bearer, id: &RawStr) -> Result<()> {
    usecases::delete_event(&mut *db, &id.to_string(), &token.0)?;
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use crate::core::entities::*;
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
            assert_eq!(response.status(), Status::Unauthorized);
            // But in the future we might allow anonymous event creation:
            //
            // assert_eq!(response.status(), Status::Ok);
            // test_json(&response);
            // let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            // let eid = db.get().unwrap().all_events().unwrap()[0].id.clone();
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
            assert_eq!(response.status(), Status::Unauthorized);
            // But in the future we might allow anonymous event creation:
            //
            // assert_eq!(response.status(), Status::Ok);
            // test_json(&response);
            // let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            // let ev = db.get().unwrap().all_events().unwrap()[0].clone();
            // let eid = ev.id.clone();
            // assert!(ev.created_by.is_none());
            // assert_eq!(body_str, format!("\"{}\"", eid));
            // let req = client
            //     .get(format!("/events/{}", eid))
            //     .header(ContentType::JSON);
            // let mut response = req.dispatch();
            // assert_eq!(response.status(), Status::Ok);
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
            fn with_creator_email() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::Ok);
                test_json(&res);
                let body_str = res.body().and_then(|b| b.into_string()).unwrap();
                let ev = db.get().unwrap().all_events().unwrap()[0].clone();
                let eid = ev.id.clone();
                assert_eq!(ev.created_by.unwrap(), "foobarcom");
                assert_eq!(body_str, format!("\"{}\"", eid));
            }

            #[test]
            fn with_empty_strings_for_optional_fields() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","email":"","homepage":"","description":"","registration":""}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::Ok);
                test_json(&res);
                let ev = db.get().unwrap().all_events().unwrap()[0].clone();
                assert!(ev.contact.is_none());
                assert!(ev.homepage.is_none());
                assert!(ev.description.is_none());
            }

            #[test]
            fn with_registration_type() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"telephone","telephone":"12345"}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::Ok);
                test_json(&res);
                let ev = db.get().unwrap().all_events().unwrap()[0].clone();
                assert_eq!(ev.registration.unwrap(), RegistrationType::Phone);
            }

            #[test]
            fn with_invalid_registration_type() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"foo"}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::BadRequest);
            }

            #[test]
            fn without_creator_email() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::BadRequest);
            }

            #[test]
            fn with_empty_title() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"","start":0,"created_by":"foo@bar.com"}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::BadRequest);
            }

            #[test]
            fn with_phone_registraion_but_without_phone_nr() {
                let (client, db) = setup();
                db.get()
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
                    .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com","registration":"telephone"}"#)
                    .dispatch();
                assert_eq!(res.status(), Status::BadRequest);
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
            assert_eq!(res.status(), Status::Unauthorized);
        }

    }

    mod read {
        use super::*;

        #[test]
        fn by_id() {
            let (client, db) = setup();
            let e = Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: 0,
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
            };
            db.get().unwrap().create_event(e).unwrap();
            let req = client.get("/events/1234").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(
                body_str,
                r#"{"id":"1234","title":"x","start":0,"tags":["bla"],"registration":"email"}"#
            );
        }

        #[test]
        fn all() {
            let (client, db) = setup();
            let event_ids = vec!["a", "b", "c"];
            let mut db = db.get().unwrap();
            for id in event_ids {
                db.create_event(Event {
                    id: id.into(),
                    title: id.into(),
                    description: None,
                    start: 0,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let req = client.get("/events").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"id\":\"a\""));
        }

        #[test]
        fn sorted_by_start() {
            let (client, db) = setup();
            let event_start_times = vec![100, 0, 300, 50, 200];
            let mut db = db.get().unwrap();
            for start in event_start_times {
                db.create_event(Event {
                    id: start.to_string(),
                    title: start.to_string(),
                    description: None,
                    start,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let mut res = client.get("/events").header(ContentType::JSON).dispatch();
            assert_eq!(res.status(), Status::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert!(objects[0].contains("\"id\":\"0\""));
            assert!(objects[1].contains("\"id\":\"50\""));
            assert!(objects[2].contains("\"id\":\"100\""));
            assert!(objects[3].contains("\"id\":\"200\""));
            assert!(objects[4].contains("\"id\":\"300\""));
        }

        #[test]
        fn filtered_by_tags() {
            let (client, db) = setup();
            let event_ids = vec!["a", "b", "c"];
            let mut db = db.get().unwrap();
            for id in event_ids {
                db.create_event(Event {
                    id: id.into(),
                    title: id.into(),
                    description: None,
                    start: 0,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![id.into()],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let req = client.get("/events?tag=a&tag=c").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"id\":\"a\""));
            assert!(!body_str.contains("\"id\":\"b\""));
            assert!(body_str.contains("\"id\":\"c\""));

            let req = client.get("/events?tag=b").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert!(!body_str.contains("\"id\":\"a\""));
            assert!(body_str.contains("\"id\":\"b\""));
            assert!(!body_str.contains("\"id\":\"c\""));
        }

        #[test]
        fn filtered_by_creator_without_api_token() {
            let (client, _db) = setup();
            let res = client
                .get("/events?created_by=foo%40bar.com")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn filtered_by_creator_with_valid_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let emails = vec!["foo@bar.com", "test@test.com", "bla@bla.bla"];
            let mut db = db.get().unwrap();
            for (i, m) in emails.into_iter().enumerate() {
                let username = m.to_string().replace(".", "").replace("@", "");
                db.create_event(Event {
                    id: i.to_string(),
                    title: m.into(),
                    description: None,
                    start: 0,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: Some(username.clone()),
                    registration: None,
                })
                .unwrap();
                db.create_user(User {
                    id: i.to_string(),
                    username,
                    password: "secret".into(),
                    email: m.into(),
                    email_confirmed: true,
                    role: Role::default(),
                })
                .unwrap();
            }
            let mut res = client
                .get("/events?created_by=test%40test.com")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            assert!(body_str.contains("\"id\":\"1\""));
            assert!(!body_str.contains("\"id\":\"0\""));
            assert!(!body_str.contains("\"id\":\"2\""));
        }

        #[test]
        fn filtered_by_creator_with_invalid_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();

            let res = client
                .get("/events?created_by=foo@bar.com")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .dispatch();
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn filtered_by_start_min() {
            let (client, db) = setup();
            let event_start_times = vec![100, 0, 300, 50, 200];
            let mut db = db.get().unwrap();
            for start in event_start_times {
                db.create_event(Event {
                    id: start.to_string(),
                    title: start.to_string(),
                    description: None,
                    start,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let mut res = client
                .get("/events?start_min=150")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 2);
            assert!(objects[0].contains("\"id\":\"200\""));
            assert!(objects[1].contains("\"id\":\"300\""));
        }

        #[test]
        fn filtered_by_start_max() {
            let (client, db) = setup();
            let event_start_times = vec![100, 0, 300, 50, 200];
            let mut db = db.get().unwrap();
            for start in event_start_times {
                db.create_event(Event {
                    id: start.to_string(),
                    title: start.to_string(),
                    description: None,
                    start,
                    end: None,
                    location: None,
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let mut res = client
                .get("/events?start_max=250")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 4);
            assert!(objects[0].contains("\"id\":\"0\""));
            assert!(objects[1].contains("\"id\":\"50\""));
            assert!(objects[2].contains("\"id\":\"100\""));
            assert!(objects[3].contains("\"id\":\"200\""));
        }

        #[test]
        fn filtered_by_bounding_box() {
            let (client, db) = setup();
            let mut db = db.get().unwrap();
            let coordinates = &[(-8.0, 0.0), (0.3, 5.0), (7.0, 7.9), (12.0, 0.0)];
            for &(lat, lng) in coordinates {
                db.create_event(Event {
                    id: format!("{}-{}", lat, lng),
                    title: format!("{}-{}", lat, lng),
                    description: None,
                    start: 0,
                    end: None,
                    location: Some(Location {
                        lat,
                        lng,
                        address: None,
                    }),
                    contact: None,
                    tags: vec![],
                    homepage: None,
                    created_by: None,
                    registration: None,
                })
                .unwrap();
            }
            let mut res = client
                .get("/events?bbox=-8,-5,10,7.9")
                .header(ContentType::JSON)
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            test_json(&res);
            let body_str = res.body().and_then(|b| b.into_string()).unwrap();
            let objects: Vec<_> = body_str.split("},{").collect();
            assert_eq!(objects.len(), 3);
            assert!(objects[0].contains("\"id\":\"-8-0\""));
            assert!(objects[1].contains("\"id\":\"0.3-5\""));
            assert!(objects[2].contains("\"id\":\"7-7.9\""));
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
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn with_invalid_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let res = client
                .put("/events/foo")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn with_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: 0,
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: Some("foo@bar.com".into()),
                registration: None,
            };
            db.get().unwrap().create_event(e.clone()).unwrap();
            let res = client
                .put("/events/1234")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new","start":5,"created_by":"changed@bar.com"}"#)
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            let new = db.get().unwrap().get_event("1234").unwrap();
            assert_eq!(&*new.title, "new");
            assert_eq!(new.start, 5);
            assert!(new.created_by != e.created_by);
        }

        #[test]
        fn with_api_token_without_created_by() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e = Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: 0,
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: Some("foo@bar.com".into()),
                registration: None,
            };
            db.get().unwrap().create_event(e.clone()).unwrap();
            let res = client
                .put("/events/1234")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body("{\"title\":\"Changed\",\"start\":99}")
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            let new = db.get().unwrap().get_event("1234").unwrap();
            assert_eq!(&*new.title, "Changed");
            assert!(new.created_by == e.created_by);
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
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn with_invalid_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let res = client
                .delete("/events/foo")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer bar"))
                .dispatch();
            assert_eq!(res.status(), Status::Unauthorized);
        }

        #[test]
        fn with_api_token() {
            let (client, db) = setup();
            db.get()
                .unwrap()
                .create_org(Organization {
                    id: "foo".into(),
                    name: "bar".into(),
                    owned_tags: vec![],
                    api_token: "foo".into(),
                })
                .unwrap();
            let e0 = Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: 0,
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: Some("foo@bar.com".into()),
                registration: None,
            };
            let e1 = Event {
                id: "9999".into(),
                title: "x".into(),
                description: None,
                start: 0,
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: Some("foo@bar.com".into()),
                registration: None,
            };
            db.get().unwrap().create_event(e0.clone()).unwrap();
            db.get().unwrap().create_event(e1.clone()).unwrap();
            assert_eq!(db.get().unwrap().all_events().unwrap().len(), 2);
            let res = client
                .delete("/events/1234")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .dispatch();
            assert_eq!(res.status(), Status::Ok);
            assert_eq!(db.get().unwrap().all_events().unwrap().len(), 1);
        }
    }

}
