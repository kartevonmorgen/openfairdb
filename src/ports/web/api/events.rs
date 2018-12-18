use super::*;

#[post("/events", format = "application/json", data = "<e>")]
pub fn post_event(mut db: DbConn, e: Json<usecases::NewEvent>) -> Result<String> {
    let e = e.into_inner();
    let id = usecases::create_new_event(&mut *db, e.clone())?;
    Ok(Json(id))
}

#[get("/events/<id>")]
pub fn get_event(mut db: DbConn, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&mut *db, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use crate::core::entities::*;

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
    fn create_event_with_creator_email() {
        let (client, db) = setup();
        let req = client
            .post("/events")
            .header(ContentType::JSON)
            .body(r#"{"title":"x","start":0,"created_by":"foo@bar.com"}"#);
        let mut response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        test_json(&response);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();
        let eid = db.get().unwrap().all_events().unwrap()[0].id.clone();
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
}
