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
    start: Option<i64>,
    end: Option<i64>,
}

impl<'q> FromQuery<'q> for EventQuery {
    type Error = Error;

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
            .filter(|i| i.key == "created_by")
            .map(|i| i.value.url_decode_lossy())
            .filter(|v| !v.is_empty())
            .nth(0);

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
    let events = usecases::query_events(&*db, query.tags, &query.created_by, Some(token.0))?;
    let events = events.into_iter().map(json::Event::from).collect();
    Ok(Json(events))
}

#[get("/events?<query..>", rank = 2)]
pub fn get_events(db: DbConn, query: EventQuery) -> Result<Vec<json::Event>> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }
    let events = usecases::query_events(&*db, query.tags, &query.created_by, None)?;
    let events = events.into_iter().map(json::Event::from).collect();
    Ok(Json(events))
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

        #[test]
        fn with_api_token_and_creator_email() {
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
            let req = client
                .post("/events")
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            let ev = db.get().unwrap().all_events().unwrap()[0].clone();
            let eid = ev.id.clone();
            assert_eq!(ev.created_by.unwrap(), "foobarcom");
            assert_eq!(body_str, format!("\"{}\"", eid));
        }

        #[test]
        fn with_api_token_and_without_creator_email() {
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
        fn with_api_token_and_with_empty_title() {
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
            };
            db.get().unwrap().create_event(e).unwrap();
            let req = client.get("/events/1234").header(ContentType::JSON);
            let mut response = req.dispatch();
            assert_eq!(response.status(), Status::Ok);
            test_json(&response);
            let body_str = response.body().and_then(|b| b.into_string()).unwrap();
            assert_eq!(
                body_str,
                r#"{"id":"1234","title":"x","start":0,"lat":0.0,"lng":0.0,"tags":["bla"]}"#
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

        #[ignore]
        #[test]
        fn filtered_by_start_time() {
            //TODO: implement
        }

        #[ignore]
        #[test]
        fn filtered_by_end_time() {
            //TODO: implement
        }

        #[ignore]
        #[test]
        fn filtered_by_bounding_box() {
            //TODO: implement
        }
    }

    mod update {
        use super::*;

        #[ignore]
        #[test]
        fn without_api_token() {
            //TODO: implement
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
    }

    mod delete {
        use super::*;

        #[ignore]
        #[test]
        fn without_api_token() {
            //TODO: implement
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
