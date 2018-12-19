use super::{super::guards::Bearer, *};

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

#[post("/events", format = "application/json", data = "<e>", rank = 2)]
pub fn post_event(mut db: DbConn, e: Json<usecases::NewEvent>) -> Result<String> {
    let mut e = e.into_inner();
    e.created_by = None; // ignore because of missing authorization
    e.token = None; // ignore token
    let id = usecases::create_new_event(&mut *db, e.clone())?;
    Ok(Json(id))
}

#[get("/events/<id>")]
pub fn get_event(db: DbConn, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&*db, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[derive(FromForm, Clone)]
pub struct EventQuery {
    tags: Option<String>,
    created_by: Option<String>,
}

#[get("/events?<query..>")]
pub fn get_events(db: DbConn, query: Form<EventQuery>) -> Result<Vec<json::Event>> {
    let mut tags = None;
    if let Some(ref tags_str) = query.tags {
        let mut res = vec![];
        for t in util::extract_ids(tags_str) {
            res.push(t);
        }
        tags = Some(res);
    }
    let events = usecases::query_events(&*db, tags, &query.created_by)?;
    let events = events.into_iter().map(json::Event::from).collect();
    Ok(Json(events))
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use crate::core::entities::*;
    use rocket::http::Header;

    #[test]
    fn create_event_without_creator_email() {
        let (client, db) = setup();
        let req = client
            .post("/events")
            .header(ContentType::JSON)
            .body(r#"{"title":"foo","start":1234}"#);
        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        test_json(&response);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let eid = db.get().unwrap().all_events().unwrap()[0].id.clone();
        assert_eq!(body_str, format!("\"{}\"", eid));
    }

    #[test]
    fn create_event_without_api_token_but_with_creator_email() {
        let (client, db) = setup();
        let req = client
            .post("/events")
            .header(ContentType::JSON)
            .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#);
        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        test_json(&response);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let ev = db.get().unwrap().all_events().unwrap()[0].clone();
        let eid = ev.id.clone();
        assert!(ev.created_by.is_none());
        assert_eq!(body_str, format!("\"{}\"", eid));
        let req = client
            .get(format!("/events/{}", eid))
            .header(ContentType::JSON);
        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        test_json(&response);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        assert_eq!(
            body_str,
            format!(
                "{{\"id\":\"{}\",\"title\":\"x\",\"start\":0,\"lat\":0.0,\"lng\":0.0,\"tags\":[]}}",
                eid
            )
        );
    }

    #[test]
    fn create_event_with_api_token_and_creator_email() {
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
    fn create_event_with_invalid_api_token() {
        let (client, _) = setup();
        let req = client
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer not-valid"))
            .body(r#"{"title":"x","start":0}"#);
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn get_event() {
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
    fn get_events() {
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
        body_str.contains("\"id\":\"a\"");
    }
}
