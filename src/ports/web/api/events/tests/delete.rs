use super::*;

#[test]
fn without_api_token() {
    let (client, _) = setup();
    let res = client
        .delete("/events/foo")
        .header(ContentType::JSON)
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
            moderated_tags: vec!["org-tag".into()],
            api_token: "foo".into(),
        })
        .unwrap();
    let res = client
        .delete("/events/foo")
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer bar"))
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
    // Manually delete the implicitly added org tag from the 2nd event!
    let mut e2 = db.shared().unwrap().get_event(id2.as_ref()).unwrap();
    e2.tags.retain(|t| t != "tag");
    db.exclusive().unwrap().update_event(&e2).unwrap();
    assert_eq!(db.shared().unwrap().count_events().unwrap(), 2);
    // The 1st event has the owned tag and should be deleted.
    let res = client
        .delete(format!("/events/{}", id1))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer foo"))
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Ok);
    // The 2nd event is not tagged with one of the owned tags.
    let res = client
        .delete(format!("/events/{}", id2))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer foo"))
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Forbidden);
    assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
}

// FIXME: This test should fail, but it doesn't!!
#[test]
#[ignore]
fn with_api_token_by_organization_without_any_moderated_tags() {
    let (client, db, mut search_engine, notify) = setup2();
    db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "foo".into(),
            name: "bar".into(),
            moderated_tags: vec![],
            api_token: "foo".into(),
        })
        .unwrap();
    let e = usecases::NewEvent {
        title: "x".into(),
        start: Utc::now().naive_utc().timestamp(),
        tags: Some(vec!["bla".into()]),
        created_by: Some("foo@bar.com".into()),
        ..Default::default()
    };
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("foo"), e)
        .unwrap()
        .id;
    assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
    let res = client
        .delete(format!("/events/{}", id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer foo"))
        .dispatch();
    assert_eq!(db.shared().unwrap().count_events().unwrap(), 1);
    assert_eq!(res.status(), HttpStatus::Unauthorized);
}

#[test]
fn with_api_token_from_different_org_unauthorized() {
    let (client, db, mut search_engine, notify) = setup2();
    let _creator_org = db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "creator".into(),
            name: "creator".into(),
            moderated_tags: vec!["creator".into()],
            api_token: "creator".into(),
        })
        .unwrap();
    let _deleter_org = db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "deleter".into(),
            name: "deleter".into(),
            moderated_tags: vec!["deleter".into()],
            api_token: "deleter".into(),
        })
        .unwrap();
    let e = usecases::NewEvent {
        title: "x".into(),
        tags: Some(vec!["bla".into(), "creator".into()]),
        created_by: Some("creator@example.com".into()),
        start: Utc::now().naive_utc().timestamp(),
        ..Default::default()
    };
    let id = flows::create_event(&db, &mut search_engine, &notify, Some("creator"), e)
        .unwrap()
        .id;
    assert!(db.shared().unwrap().get_event(id.as_ref()).is_ok());
    // Try to delete the event using the token of another organization.
    let res = client
        .delete(format!("/events/{}", id))
        .header(ContentType::JSON)
        .header(Header::new("Authorization", "Bearer deleter"))
        .dispatch();
    assert_eq!(res.status(), HttpStatus::Forbidden);
}
