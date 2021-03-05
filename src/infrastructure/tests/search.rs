use super::*;

#[test]
fn should_find_places_by_tags() -> flows::Result<()> {
    let fixture = flows::BackendFixture::new();

    let place_without_tags = flows::create_place(
        &fixture.db_connections,
        &mut *fixture.search_engine.borrow_mut(),
        &fixture.notify,
        usecases::NewPlace {
            title: "place".into(),
            description: "place".into(),
            ..default_new_place()
        },
        None,
        None,
        &Cfg::default(),
    )
    .unwrap();
    assert!(place_without_tags.tags.is_empty());

    let place_foo = flows::create_place(
        &fixture.db_connections,
        &mut *fixture.search_engine.borrow_mut(),
        &fixture.notify,
        usecases::NewPlace {
            title: "place_foo".into(),
            description: "place_foo".into(),
            tags: vec!["foo".to_string()],
            ..default_new_place()
        },
        None,
        None,
        &Cfg::default(),
    )
    .unwrap();

    let place_bar = flows::create_place(
        &fixture.db_connections,
        &mut *fixture.search_engine.borrow_mut(),
        &fixture.notify,
        usecases::NewPlace {
            title: "place_without_tags".into(),
            description: "place_without_tags".into(),
            tags: vec!["bar".to_string()],
            ..default_new_place()
        },
        None,
        None,
        &Cfg::default(),
    )
    .unwrap();

    let place_foo_and_bar = flows::create_place(
        &fixture.db_connections,
        &mut *fixture.search_engine.borrow_mut(),
        &fixture.notify,
        usecases::NewPlace {
            title: "place_without_tags".into(),
            description: "place_without_tags".into(),
            tags: vec!["foo".to_string(), "bar".to_string()],
            ..default_new_place()
        },
        None,
        None,
        &Cfg::default(),
    )
    .unwrap();

    let place_foo_hyphen_bar = flows::create_place(
        &fixture.db_connections,
        &mut *fixture.search_engine.borrow_mut(),
        &fixture.notify,
        usecases::NewPlace {
            title: "place_without_tags".into(),
            description: "place_without_tags".into(),
            tags: vec!["foo-bar".to_string()],
            ..default_new_place()
        },
        None,
        None,
        &Cfg::default(),
    )
    .unwrap();

    let search_foo_ids: Vec<Id> = usecases::search(
        &*fixture.db_connections.shared()?,
        &*fixture.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec!["foo"],
            ..default_search_request()
        },
        100,
    )?
    .0
    .into_iter()
    .map(|p| p.id.into())
    .collect();
    assert_eq!(2, search_foo_ids.len());
    assert!(!search_foo_ids.contains(&place_without_tags.id));
    assert!(search_foo_ids.contains(&place_foo.id));
    assert!(!search_foo_ids.contains(&place_bar.id));
    assert!(search_foo_ids.contains(&place_foo_and_bar.id));
    assert!(!search_foo_ids.contains(&place_foo_hyphen_bar.id));

    let search_bar_ids: Vec<Id> = usecases::search(
        &*fixture.db_connections.shared()?,
        &*fixture.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec!["bar"],
            ..default_search_request()
        },
        100,
    )?
    .0
    .into_iter()
    .map(|p| p.id.into())
    .collect();
    assert_eq!(2, search_bar_ids.len());
    assert!(!search_bar_ids.contains(&place_without_tags.id));
    assert!(!search_bar_ids.contains(&place_foo.id));
    assert!(search_bar_ids.contains(&place_bar.id));
    assert!(search_bar_ids.contains(&place_foo_and_bar.id));
    assert!(!search_bar_ids.contains(&place_foo_hyphen_bar.id));

    let search_foo_and_bar_ids: Vec<Id> = usecases::search(
        &*fixture.db_connections.shared()?,
        &*fixture.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec!["foo", "bar"],
            ..default_search_request()
        },
        100,
    )?
    .0
    .into_iter()
    .map(|p| p.id.into())
    .collect();
    assert_eq!(1, search_foo_and_bar_ids.len());
    assert!(!search_foo_and_bar_ids.contains(&place_without_tags.id));
    assert!(!search_foo_and_bar_ids.contains(&place_foo.id));
    assert!(!search_foo_and_bar_ids.contains(&place_bar.id));
    assert!(search_foo_and_bar_ids.contains(&place_foo_and_bar.id));
    assert!(!search_foo_and_bar_ids.contains(&place_foo_hyphen_bar.id));

    // TODO: Search tags by prefix with a wildcard suffix, e.g. "foo*"" should
    // match both "foo" and "foo-bar"? This feature is currently not supported
    // by Tantivy, i.e. only tags that equal the search term (case insensitive)
    // are found.

    Ok(())
}
