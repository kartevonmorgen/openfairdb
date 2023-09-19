use rocket::http::StatusClass;
use serde_json::json;
use time::Duration;

use super::*;

#[test]
fn by_id() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = Timestamp::now();
    let mut e = usecases::NewEvent::new("x".into(), now);
    e.tags = Some(vec!["bla".into()]);
    e.registration = Some("email".into());
    e.email = Some("test@example.com".parse().unwrap());
    e.created_by = Some("test@example.com".parse().unwrap());
    let e = flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    let req = client
        .get(format!("/events/{}", e.id))
        .header(ContentType::JSON);
    let response = req.dispatch();
    assert_eq!(response.status(), HttpStatus::Ok);
    test_json(&response);
    let body_string = response.into_string().unwrap();
    let json_body = serde_json::from_str::<serde_json::Value>(&body_string).unwrap();
    let expected_json = json!({
      "id": e.id.to_string(),
      "title":"x",
      "start": now.as_secs(),
      "email":"test@example.com",
      "tags":["bla"],
      "registration":"email"
    });
    assert_eq!(json_body, expected_json);
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
    let now = Timestamp::now();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + Duration::seconds(start_offset);
        let mut e = usecases::NewEvent::new(start_offset.to_string(), start);
        e.created_by = Some("test@example.com".parse().unwrap());
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    }
    let res = client.get("/events").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert!(objects[0].contains(&format!("\"start\":{}", now.as_secs())));
    assert!(objects[1].contains(&format!("\"start\":{}", now.as_secs() + 50)));
    assert!(objects[2].contains(&format!("\"start\":{}", now.as_secs() + 100)));
    assert!(objects[3].contains(&format!("\"start\":{}", now.as_secs() + 200)));
    assert!(objects[4].contains(&format!("\"start\":{}", now.as_secs() + 300)));
}

#[test]
fn filtered_by_tags() {
    let (client, db, mut search_engine, notify) = setup2();
    let tags = vec![vec!["a"], vec!["b"], vec!["c"], vec!["a", "b"]];
    for tags in tags {
        let e = usecases::NewEvent {
            title: format!("{:?}", tags),
            start: Timestamp::now(),
            tags: Some(tags.into_iter().map(str::to_string).collect()),
            created_by: Some("test@example.com".parse().unwrap()),
            ..Default::default()
        };
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
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
                created_by: Some(m.parse::<EmailAddress>().unwrap()),
                start: Timestamp::now(),
                ..Default::default()
            };
            flows::create_event(&db, &mut *search_engine, &notify, Some("foo"), new_event)
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
    let now = Timestamp::now();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + Duration::seconds(start_offset);
        let mut e = usecases::NewEvent::new(start_offset.to_string(), start);
        e.created_by = Some("test@example.com".parse().unwrap());
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?start_min={}", now.as_secs() + 150))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 2);
    assert!(objects[0].contains(&format!("\"start\":{}", now.as_secs() + 200)));
    assert!(objects[1].contains(&format!("\"start\":{}", now.as_secs() + 300)));
}

#[test]
fn dont_accept_invalid_timestamps() {
    let (client, _, _, _) = setup2();
    let maximum_timestamp_value = 253402300799_i64;
    let invalid_timestamp_value = maximum_timestamp_value + 1;

    // TODO:
    // Make sure there is a helpful error response message.
    let params = ["start_min", "start_max", "end_min", "end_max"];
    for param in params {
        let res = client
            .get(format!("/events?{param}={invalid_timestamp_value}"))
            .header(ContentType::JSON)
            .dispatch();
        assert_eq!(res.status().class(), StatusClass::ClientError);
    }
}

#[test]
fn filtered_by_end_min() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = Timestamp::now();
    let end_offsets = vec![100, 1, 300, 50, 200];
    for (start_offset, end_offset) in end_offsets.into_iter().enumerate() {
        // Differing start dates are required for ordering of search results!
        let start = now + Duration::seconds(start_offset as i64);
        let end = Some(now + Duration::seconds(end_offset));
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            end,
            created_by: Some("test@example.com".parse().unwrap()),
            ..Default::default()
        };
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?end_min={}", now.as_secs() + 150))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 2);
    assert!(objects[0].contains(&format!("\"end\":{}", now.as_secs() + 300)));
    assert!(objects[1].contains(&format!("\"end\":{}", now.as_secs() + 200)));
}

#[test]
fn filtered_by_start_max() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = Timestamp::now();
    let start_offsets = vec![100, 0, 300, 50, 200];
    for start_offset in start_offsets {
        let start = now + Duration::seconds(start_offset);
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            created_by: Some("test@example.com".parse().unwrap()),
            ..Default::default()
        };
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?start_max={}", now.as_secs() + 250))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 4);
    assert!(objects[0].contains(&format!("\"start\":{}", now.as_secs())));
    assert!(objects[1].contains(&format!("\"start\":{}", now.as_secs() + 50)));
    assert!(objects[2].contains(&format!("\"start\":{}", now.as_secs() + 100)));
    assert!(objects[3].contains(&format!("\"start\":{}", now.as_secs() + 200)));
}

#[test]
fn filtered_by_end_max() {
    let (client, db, mut search_engine, notify) = setup2();
    let now = Timestamp::now();
    let end_offsets = vec![100, 1, 300, 50, 200];
    for (start_offset, end_offset) in end_offsets.into_iter().enumerate() {
        // Differing start dates are required for ordering of search results!
        let start = now + Duration::seconds(start_offset as i64);
        let end = Some(now + Duration::seconds(end_offset));
        let e = usecases::NewEvent {
            title: start.to_string(),
            start,
            end,
            created_by: Some("test@example.com".parse().unwrap()),
            ..Default::default()
        };
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
    }
    let res = client
        .get(format!("/events?end_max={}", now.as_secs() + 250))
        .header(ContentType::JSON)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let objects: Vec<_> = body_str.split("},{").collect();
    assert_eq!(objects.len(), 4);
    assert!(objects[0].contains(&format!("\"end\":{}", now.as_secs() + 100)));
    assert!(objects[1].contains(&format!("\"end\":{}", now.as_secs() + 1)));
    assert!(objects[2].contains(&format!("\"end\":{}", now.as_secs() + 50)));
    assert!(objects[3].contains(&format!("\"end\":{}", now.as_secs() + 200)));
}

#[test]
fn filtered_by_bounding_box() {
    let (client, db, mut search_engine, notify) = setup2();
    let coordinates = &[(-8.0, 0.0), (0.3, 5.0), (7.0, 7.9), (12.0, 0.0)];
    for &(lat, lng) in coordinates {
        let e = usecases::NewEvent {
            title: format!("{}-{}", lat, lng),
            start: Timestamp::now(),
            lat: Some(lat),
            lng: Some(lng),
            created_by: Some("test@example.com".parse().unwrap()),
            ..Default::default()
        };
        flows::create_event(&db, &mut *search_engine, &notify, None, e).unwrap();
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
