use super::*;

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
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
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
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
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
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("bar"), e)
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
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
        .unwrap()
        .id;
    assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
    let res = client
        .put(format!("/events/{}", id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer foo"))
        .body(
            r#"{"title":"new","start":4132508400,"created_by":"changed@bar.com","tags":["bla2"]}"#,
        )
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    let new = db.exclusive().unwrap().get_event(id.as_ref()).unwrap();
    assert_eq!(new.tags, vec!["bla2", "org-tag"]);
}

#[test]
fn with_api_token_and_removing_tag() {
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
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
    let (client, db, mut search_engine, notify) = setup2();
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
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
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
        .body(&format!(
            "{{\"title\":\"Changed again\",\"created_by\":\"changed@bar.com\",\"start\":{}}}",
            start
        ))
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    let new = db.shared().unwrap().get_event(id.as_ref()).unwrap();
    assert_eq!(new.title, "Changed again");
    // created_by has been updated
    assert_eq!(new.created_by, Some("changed@bar.com".into()));
}

#[test]
fn update_geo_location() {
    let (client, db, mut search_engine, notify) = setup2();
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
        lat: Some(1.0),
        lng: Some(2.0),
        ..Default::default()
    };
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
        .unwrap()
        .id;
    let created = db.shared().unwrap().get_event(id.as_ref()).unwrap();
    assert_eq!(
        Some((
            LatCoord::from_deg(1.0).to_deg(),
            LngCoord::from_deg(2.0).to_deg()
        )),
        created.location.map(|loc| loc.pos.to_lat_lng_deg())
    );
    let res = client
                .put(format!("/events/{}", id))
                .header(ContentType::JSON)
                .header(Header::new("Authorization", "Bearer foo"))
                .body(r#"{"title":"new title","start":4132508400,"created_by":"updated@example.com","lat":-1.0,"lng":-2.0}"#)
                .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    let updated = db.shared().unwrap().get_event(id.as_ref()).unwrap();
    assert_eq!(
        Some((
            LatCoord::from_deg(-1.0).to_deg(),
            LngCoord::from_deg(-2.0).to_deg()
        )),
        updated.location.map(|loc| loc.pos.to_lat_lng_deg())
    );
}
