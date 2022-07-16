use super::*;

// Magic time constant: 4132508400 ~= 2030-01-01 00:00:00 + 70 years

#[test]
fn without_creator_email() {
    let (client, _db) = setup();
    let req = client
        .post("/events")
        .header(ContentType::JSON)
        .body(r#"{"title":"foo","start":4132508400}"#);
    let response = req.dispatch();

    // NOTE:
    // At the moment we don't want to allow anonymous event creation.
    // So for now we assure that it's blocked:
    assert_eq!(response.status(), HttpStatus::Unauthorized);
    // But in the future we might allow anonymous event creation:
    //
    // assert_eq!(response.status(), HttpStatus::Ok);
    // test_json(&response);
    // let body_str = response.into_string().unwrap();
    // let eid = db.get().unwrap().all_events_chronologically().unwrap()[0].id.
    // clone(); assert_eq!(body_str, format!("\"{}\"", eid));
}

#[test]
fn without_api_token_but_with_creator_email() {
    let (client, _db) = setup();
    let req = client
        .post("/events")
        .header(ContentType::JSON)
        .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#);
    let response = req.dispatch();
    // NOTE:
    // At the moment we don't want to allow anonymous event creation.
    // So for now we assure that it's blocked:
    assert_eq!(response.status(), HttpStatus::Unauthorized);
    // But in the future we might allow anonymous event creation:
    //
    // assert_eq!(response.status(), HttpStatus::Ok);
    // test_json(&response);
    // let body_str = response.into_string().unwrap();
    // let ev = db.get().unwrap().all_events_chronologically().unwrap()[0].
    // clone(); let eid = ev.id.clone();
    // assert!(ev.created_by.is_none());
    // assert_eq!(body_str, format!("\"{}\"", eid));
    // let req = client
    //     .get(format!("/events/{}", eid))
    //     .header(ContentType::JSON);
    // let response = req.dispatch();
    // assert_eq!(response.status(), HttpStatus::Ok);
    // test_json(&response);
    // let body_str = response.into_string().unwrap();
    // assert_eq!(
    //     body_str,
    //     format!(
    //         "{{\"id\":\"{}\",\"title\":\"x\",\"start\":0,\"lat\":0.0,\"lng\":
    // 0.0,\"tags\":[]}}",         eid
    //     )
    // );
}

mod with_api_token {
    use super::*;

    #[test]
    fn for_organization_without_any_moderated_tags() {
        let (client, db) = setup();
        db.exclusive()
            .unwrap()
            .create_org(Organization {
                id: "foo".into(),
                name: "bar".into(),
                moderated_tags: vec![],
                api_token: "foo".into(),
            })
            .unwrap();
        let res = client
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer foo"))
            .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let body_str = res.into_string().unwrap();
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        let eid = ev.id.clone();
        assert_eq!(ev.created_by.unwrap(), "foo@bar.com");
        assert_eq!(body_str, format!("\"{}\"", eid));
    }

    #[test]
    fn with_creator_email() {
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
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer foo"))
            .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com"}"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let body_str = res.into_string().unwrap();
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        let eid = ev.id.clone();
        assert_eq!(ev.created_by.unwrap(), "foo@bar.com");
        assert_eq!(body_str, format!("\"{}\"", eid));
    }

    #[test]
    fn with_a_very_long_email() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"Reginaltreffen","start":4132508400,"created_by":"a-very-super-long-email-address@a-super-long-domain.com"}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        let u = db.shared().unwrap().all_users().unwrap()[0].clone();
        assert_eq!(
            u.email,
            "a-very-super-long-email-address@a-super-long-domain.com"
        );
    }

    #[test]
    fn with_empty_strings_for_optional_fields() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","email":"","homepage":"","description":"","registration":""}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        assert!(ev.contact.is_none());
        assert!(ev.homepage.is_none());
        assert!(ev.description.is_none());
    }

    #[test]
    fn with_negative_start_end() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"title","description":"","start":-4132508400,"end":-4132508399,"created_by":"foo@bar.com"}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        assert_eq!(NaiveDateTime::from_timestamp(-4132508400, 0), ev.start);
        assert_eq!(Some(NaiveDateTime::from_timestamp(-4132508399, 0)), ev.end);
    }

    #[test]
    fn with_registration_type() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"telephone","telephone":"12345"}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        assert_eq!(ev.registration.unwrap(), RegistrationType::Phone);
    }

    #[test]
    fn with_reseved_tag_from_foreign_org() {
        let (client, db) = setup();
        db.exclusive()
            .unwrap()
            .create_org(Organization {
                id: "a".into(),
                name: "a".into(),
                moderated_tags: vec!["a".into()],
                api_token: "a".into(),
            })
            .unwrap();
        db.exclusive()
            .unwrap()
            .create_org(Organization {
                id: "b".into(),
                name: "b".into(),
                moderated_tags: vec!["b".into()],
                api_token: "b".into(),
            })
            .unwrap();
        let res = client
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer a"))
            .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["a"] }"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        let res = client
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer a"))
            .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["b"] }"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::Forbidden);
        let res = client
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer b"))
            .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["b"] }"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
    }

    #[test]
    fn with_spaces_in_tags() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","tags":["", " "," tag","tag ","two tags", "tag"]}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::Ok);
        test_json(&res);
        let ev = db.shared().unwrap().all_events_chronologically().unwrap()[0].clone();
        let mut actual_tags = ev.tags;
        actual_tags.sort_unstable();
        let mut expected_tags = vec![
            "tag".to_string(),
            "tags".to_string(),
            "two".to_string(),
            // Including the implicitly added org tag
            "org-tag".to_string(),
        ];
        expected_tags.sort_unstable();
        assert_eq!(expected_tags, actual_tags);
    }

    #[test]
    fn with_invalid_registration_type() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"foo"}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::BadRequest);
    }

    #[test]
    fn without_creator_email() {
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
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer foo"))
            .body(r#"{"title":"x","start":4132508400}"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::BadRequest);
    }

    #[test]
    fn with_empty_title() {
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
            .post("/events")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", "Bearer foo"))
            .body(r#"{"title":"","start":4132508400,"created_by":"foo@bar.com"}"#)
            .dispatch();
        assert_eq!(res.status(), HttpStatus::BadRequest);
    }

    #[test]
    fn with_phone_registration_but_without_phone_nr() {
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
                    .post("/events")
                    .header(ContentType::JSON)
                    .header(Header::new("Authorization", "Bearer foo"))
                    .body(r#"{"title":"x","start":4132508400,"created_by":"foo@bar.com","registration":"telephone"}"#)
                    .dispatch();
        assert_eq!(res.status(), HttpStatus::BadRequest);
    }
}

#[test]
fn with_invalid_api_token() {
    let (client, _) = setup();
    let res = client
        .post("/events")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer not-valid"))
        .body(r#"{"title":"x","start":4132508400}"#)
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Unauthorized);
}
