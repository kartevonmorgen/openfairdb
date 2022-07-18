use super::*;

#[test]
fn by_id() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = now();
    let e = usecases::NewEvent {
        title: "x".into(),
        start: now,
        tags: Some(vec!["bla".into()]),
        registration: Some("email".into()),
        email: Some("test@example.com".into()),
        created_by: Some("test@example.com".into()),
        ..Default::default()
    };
    let e = flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    let req = client
        .get(format!("/events/{}", e.id))
        .header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
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
                start: Timestamp::now(),
                end: None,
                location: None,
                contact: None,
                tags: vec![],
                homepage: None,
                created_by: None,
                registration: None,
                archived: None,
                image_url: None,
                image_link_url: None,
            })
            .unwrap();
    }
    let req = client.get("/events").header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains("\"id\":\"a\""));
}

#[test]
fn sorted_by_start() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = Timestamp::now().into_seconds();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + start_offset;
        let e = usecases::NewEvent {
            title: start_offset.to_string(),
            start,
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client.get("/events").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert!(objects[0].contains(&format!("\"start\":{}", now)));
    assert!(objects[1].contains(&format!("\"start\":{}", now + 50)));
    assert!(objects[2].contains(&format!("\"start\":{}", now + 100)));
    assert!(objects[3].contains(&format!("\"start\":{}", now + 200)));
    assert!(objects[4].contains(&format!("\"start\":{}", now + 300)));
}

#[test]
fn filtered_by_tags() {
    let (client, db, mut search_engine, notify) = setup2();
    let tags = vec![vec!["a"], vec!["b"], vec!["c"], vec!["a", "b"]];
    for tags in tags {
        let e = usecases::NewEvent {
            title: format!("{:?}", tags),
            start: now(),
            tags: Some(tags.into_iter().map(str::to_string).collect()),
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }

    let req = client.get("/events?tag=a").header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains("\"tags\":[\"a\"]"));
    assert!(!body_str.contains("\"tags\":[\"b\"]"));
    assert!(!body_str.contains("\"tags\":[\"c\"]"));
    assert!(body_str.contains("\"tags\":[\"a\",\"b\"]"));

    let req = client.get("/events?tag=b").header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains("\"tags\":[\"a\"]"));
    assert!(body_str.contains("\"tags\":[\"b\"]"));
    assert!(!body_str.contains("\"tags\":[\"c\"]"));
    assert!(body_str.contains("\"tags\":[\"a\",\"b\"]"));

    let req = client.get("/events?tag=c").header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains("\"tags\":[\"a\"]"));
    assert!(!body_str.contains("\"tags\":[\"b\"]"));
    assert!(body_str.contains("\"tags\":[\"c\"]"));
    assert!(!body_str.contains("\"tags\":[\"a\",\"b\"]"));

    let req = client.get("/events?tag=a&tag=b").header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    let body_str = response.into_string().unwrap();
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
    let (client, db, mut search_engine, notify) = setup2();
    db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "foo".into(),
            name: "bar".into(),
            moderated_tags: vec!["org-tag".into()],
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
                start: now(),
                ..Default::default()
            };
            flows::create_event(&db, &mut search_engine, &notify, Some("foo"), new_event)
                .unwrap()
                .id
        })
        .collect();
    let res = client
        .get("/events?created_by=test%40test.com")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer foo"))
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    let body_str = res.into_string().unwrap();
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
            moderated_tags: vec!["org-tag".into()],
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
    let (client, db, mut search_engine, notify) = setup2();
    let now = now();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + start_offset;
        let e = usecases::NewEvent {
            title: start_offset.to_string(),
            start,
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?start_min={}", now + 150))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 2);
    assert!(objects[0].contains(&format!("\"start\":{}", now + 200)));
    assert!(objects[1].contains(&format!("\"start\":{}", now + 300)));
}

#[test]
fn filtered_by_end_min() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = now();
    let end_offsets = vec![100, 1, 300, 50, 200];
    for (start_offset, end_offset) in end_offsets.into_iter().enumerate() {
        // Differing start dates are required for ordering of search results!
        let start = now + start_offset as i64;
        let end = Some(now + end_offset);
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            end,
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?end_min={}", now + 150))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 2);
    assert!(objects[0].contains(&format!("\"end\":{}", now + 300)));
    assert!(objects[1].contains(&format!("\"end\":{}", now + 200)));
}

#[test]
fn filtered_by_start_max() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = now();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + start_offset;
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?start_max={}", now + 250))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 4);
    assert!(objects[0].contains(&format!("\"start\":{}", now)));
    assert!(objects[1].contains(&format!("\"start\":{}", now + 50)));
    assert!(objects[2].contains(&format!("\"start\":{}", now + 100)));
    assert!(objects[3].contains(&format!("\"start\":{}", now + 200)));
}

#[test]
fn filtered_by_end_max() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = now();
    let end_offsets = vec![100, 1, 300, 50, 200];
    for (start_offset, end_offset) in end_offsets.into_iter().enumerate() {
        // Differing start dates are required for ordering of search results!
        let start = now + start_offset as i64;
        let end = Some(now + end_offset);
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            end,
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?end_max={}", now + 250))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 4);
    assert!(objects[0].contains(&format!("\"end\":{}", now + 100)));
    assert!(objects[1].contains(&format!("\"end\":{}", now + 1)));
    assert!(objects[2].contains(&format!("\"end\":{}", now + 50)));
    assert!(objects[3].contains(&format!("\"end\":{}", now + 200)));
}

#[test]
fn filtered_by_bounding_box() {
    let (client, db, mut search_engine, notify) = setup2();
    let coordinates = &[(-8.0, 0.0), (0.3, 5.0), (7.0, 7.9), (12.0, 0.0)];
    for &(lat, lng) in coordinates {
        let e = usecases::NewEvent {
            title: format!("{}-{}", lat, lng),
            start: now(),
            lat: Some(lat),
            lng: Some(lng),
            created_by: Some("test@example.com".into()),
            ..Default::default()
        };
        flows::create_event(&db, &mut search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get("/events?bbox=-8,-5,10,7.9")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    assert!(body_str.contains("\"title\":\"-8-0\""));
    assert!(body_str.contains("\"title\":\"7-7.9\""));
    assert!(body_str.contains("\"title\":\"0.3-5\""));
    assert!(!body_str.contains("\"title\":\"12-0\""));

    let res = client
        .get("/events?bbox=10,-1,13,1")
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    assert!(!body_str.contains("\"title\":\"-8-0\""));
    assert!(!body_str.contains("\"title\":\"7-7.9\""));
    assert!(!body_str.contains("\"title\":\"0.3-5\""));
    assert!(body_str.contains("\"title\":\"12-0\""));
}
