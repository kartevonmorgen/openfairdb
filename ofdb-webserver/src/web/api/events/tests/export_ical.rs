use super::*;

#[test]
fn export_ical() {
    let (client, db, mut search_engine, notify) = setup2();

    let users = vec![
        User {
            email: "scout@example.com".parse().unwrap(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Scout,
        },
        User {
            email: "user@example.com".parse().unwrap(),
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
    let start1 = Timestamp::now();
    let e1 = usecases::NewEvent {
        title: "title1".into(),
        start: start1,
        tags: Some(vec!["bla".into()]), // org tag will be added implicitly!
        created_by: Some("createdby1@example.com".parse().unwrap()),
        email: Some("email1@example.com".parse().unwrap()),
        organizer: Some("Contact Name".into()),
        telephone: Some("phone1".into()),
        city: Some("Stuttgart".into()),
        street: Some("Königsstr. 1".into()),
        zip: Some("70000".into()),
        lat: Some(22.21),
        lng: Some(-133.31),
        state: Some("state".into()),
        ..Default::default()
    };
    let id1 = flows::create_event(&db, &mut *search_engine, &notify, Some("foo"), e1)
        .unwrap()
        .id;
    let start2 = Timestamp::now();
    let e2 = usecases::NewEvent {
        title: "title2".into(),
        start: start2,
        tags: Some(vec!["bli".into()]), // org tag will be added implicitly!
        created_by: Some("createdby2@example.com".parse().unwrap()),
        email: Some("email2@example.com".parse().unwrap()),
        telephone: Some("phone2".into()),
        ..Default::default()
    };
    let id2 = flows::create_event(&db, &mut *search_engine, &notify, Some("bar"), e2)
        .unwrap()
        .id;

    let response = client.get("/export/events.ical").dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    // Regular users are not allowed to export events
    let login = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "user@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(login.status(), Status::Ok);
    let req = client.get("/export/events.ical");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);

    // Scout without token sees contact details of all events, but not created_by
    let login = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(login.status(), Status::Ok);
    let response = client.get("/export/events.ical").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    eprintln!("{}", body_str);

    assert!(body_str.starts_with("BEGIN:VCALENDAR\r\nVERSION:2.0"));
    assert!(body_str.ends_with("END:VCALENDAR\r\n"));
    assert!(body_str.contains(&format!("UID:{}\r\n", id1)));
    assert!(body_str.contains("SUMMARY:title1\r\n"));
    assert!(body_str.contains(r#"LOCATION:Königsstr. 1\\\, 70000\\\, Stuttgart\\\, state"#));
    assert!(body_str.contains("GEO:22.2"));
    assert!(body_str.contains(";-133.3"));
    assert!(body_str.contains(&format!(
        "DTSTART:{}\r\n",
        chrono::NaiveDateTime::from_timestamp_opt(start1.as_secs(), 0)
            .unwrap()
            .format("%Y%m%dT%H%M%S")
    )));
    assert!(body_str.contains("CATEGORIES:bla\\,tag\r\n"));
    assert!(body_str.contains(r#"CONTACT:Contact Name\\\, email1@example.com\\\, phone1"#));
    assert!(body_str.contains(&format!("UID:{id2}\r\n")));
    assert!(body_str.contains("SUMMARY:title2\r\n"));
    assert!(body_str.contains(&format!(
        "DTSTART:{}\r\n",
        chrono::NaiveDateTime::from_timestamp_opt(start2.as_secs(), 0)
            .unwrap()
            .format("%Y%m%dT%H%M%S")
    )));
    assert!(body_str.contains("CATEGORIES:bli\\,tag2\r\n"));
    assert!(body_str.contains(r#"CONTACT:email2@example.com\\\, phone2"#));
}
