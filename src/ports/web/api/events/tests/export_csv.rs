use rocket::http::Header;

use super::*;

#[test]
fn export_csv() {
    let (client, db, mut search_engine, notify) = setup2();

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
            moderated_tags: vec!["tag".into()],
            api_token: "foo".into(),
        })
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "bar".into(),
            name: "bar_name".into(),
            moderated_tags: vec!["tag2".into()],
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
    let id1 = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e1)
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
    let id2 = flows::create_event(&db, &mut search_engine, &notify, Some("bar"), e2)
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
    let response = client.get("/export/events.csv").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
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

    // Scout with token sees contact details of all events and created_by for their
    // owned events
    let login = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(login.status(), Status::Ok);
    let response = client
        .get("/export/events.csv")
        .header(Header::new("Authorization", "Bearer foo"))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
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
    let response = client.get("/export/events.csv").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.starts_with("id,created_by,organizer,title,description,start,end,lat,lng,street,zip,city,country,state,email,phone,homepage,image_url,image_link_url,tags\n"));
    assert!(body_str.contains(&format!("{},createdby1@example.com,,title1,,{},,,,,,,,state,email1@example.com,phone1,,,,\"bla,tag\"\n", id1, start1)));
    assert!(body_str.contains(&format!(
        "{},createdby2@example.com,,title2,,{},,,,,,,,,email2@example.com,phone2,,,,\"bli,tag2\"\n",
        id2, start2
    )));
}
