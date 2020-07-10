use super::*;

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
    let (client, db, mut search_engine, notify) = setup2();

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
            moderated_tags: vec!["tag".into()],
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
    let id1 = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e1)
        .unwrap()
        .id;
    let e2 = usecases::NewEvent {
        title: "x".into(),
        start: Utc::now().naive_utc().timestamp(),
        tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
        created_by: Some("foo@bar.com".into()),
        ..Default::default()
    };
    let id2 = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e2)
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
