use super::*;

pub struct PlaceClearanceFixture {
    backend: flows::BackendFixture,

    user_email: Email,

    // A place without any tags that has been newly created, i.e. initial revision
    created_place: Place,

    // A place without any tags that has been archived
    archived_place: Place,

    // A place with ALL moderated tags that has been confirmed
    confirmed_place: Place,

    // Organization with a moderated tag that allows only add
    // and requires clearance
    organization_with_add_clearance_tag: Organization,

    // Organization with a moderated tag that allows only add
    // and requires clearance
    organization_with_remove_clearance_tag: Organization,

    // Organization with a moderated tag that allows both add
    // and remove and requires clearance
    organization_with_add_remove_clearance_tag: Organization,
}

impl PlaceClearanceFixture {
    pub fn new() -> Self {
        let backend = flows::BackendFixture::new();

        let user_email = Email::from("user@example.com".to_string());
        usecases::register_with_email(
            &mut *backend.db_connections.exclusive().unwrap(),
            &usecases::Credentials {
                email: &user_email,
                password: "password",
            },
        )
        .unwrap();

        // Create places
        let created_place = flows::create_place(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &backend.notify,
            usecases::NewPlace {
                title: "created_place".into(),
                description: "created_place".into(),
                ..default_new_place()
            },
            None,
            None,
        )
        .unwrap();

        let archived_place = flows::create_place(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &backend.notify,
            usecases::NewPlace {
                title: "archived_place".into(),
                description: "archived_place".into(),
                ..default_new_place()
            },
            None,
            None,
        )
        .unwrap();
        flows::review_places(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &[archived_place.id.as_str()],
            usecases::Review {
                status: ReviewStatus::Archived,
                reviewer_email: user_email.clone(),
                comment: None,
                context: None,
            },
        )
        .unwrap();

        let rejected_place = flows::create_place(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &backend.notify,
            usecases::NewPlace {
                title: "rejected_place".into(),
                description: "rejected_place".into(),
                ..default_new_place()
            },
            None,
            None,
        )
        .unwrap();
        flows::review_places(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &[rejected_place.id.as_str()],
            usecases::Review {
                status: ReviewStatus::Archived,
                reviewer_email: user_email.clone(),
                comment: None,
                context: None,
            },
        )
        .unwrap();

        // Prepate organizations
        let organization_without_moderated_tags = Organization {
            id: Id::new(),
            name: "organization_without_moderated_tags".into(),
            api_token: "organization_without_moderated_tags".into(),
            moderated_tags: vec![],
        };
        let organization_with_add_clearance_tag = Organization {
            id: Id::new(),
            name: "organization_with_add_clearance_tag".into(),
            api_token: "organization_with_add_clearance_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "add_clearance".into(),
                allow_add: true,
                allow_remove: false,
                require_clearance: true,
            }],
        };
        let organization_with_remove_clearance_tag = Organization {
            id: Id::new(),
            name: "organization_with_remove_clearance_tag".into(),
            api_token: "organization_with_remove_clearance_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "remove_clearance".into(),
                allow_add: false,
                allow_remove: true,
                require_clearance: true,
            }],
        };
        let organization_with_add_remove_clearance_tag = Organization {
            id: Id::new(),
            name: "organization_with_add_remove_clearance_tag".into(),
            api_token: "organization_with_add_remove_clearance_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "add_remove_clearance".into(),
                allow_add: true,
                allow_remove: true,
                require_clearance: true,
            }],
        };

        let confirmed_tags = organization_with_add_clearance_tag
            .moderated_tags
            .iter()
            .map(|mod_tag| mod_tag.label.clone())
            .chain(
                organization_with_remove_clearance_tag
                    .moderated_tags
                    .iter()
                    .map(|mod_tag| mod_tag.label.clone()),
            )
            .chain(
                organization_with_add_remove_clearance_tag
                    .moderated_tags
                    .iter()
                    .map(|mod_tag| mod_tag.label.clone()),
            )
            .collect();
        let confirmed_place = flows::create_place(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &backend.notify,
            usecases::NewPlace {
                title: "confirmed_place".into(),
                description: "confirmed_place".into(),
                tags: confirmed_tags,
                ..default_new_place()
            },
            None,
            None,
        )
        .unwrap();
        flows::review_places(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &[confirmed_place.id.as_str()],
            usecases::Review {
                status: ReviewStatus::Confirmed,
                reviewer_email: user_email.clone(),
                comment: None,
                context: None,
            },
        )
        .unwrap();

        // Insert organizations into database
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_without_moderated_tags.clone())
            .unwrap();
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_add_clearance_tag.clone())
            .unwrap();
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_remove_clearance_tag.clone())
            .unwrap();
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_add_remove_clearance_tag.clone())
            .unwrap();
        Self {
            backend,
            user_email,
            created_place,
            archived_place,
            confirmed_place,
            organization_with_add_clearance_tag,
            organization_with_remove_clearance_tag,
            organization_with_add_remove_clearance_tag,
        }
    }
}

#[test]
fn should_create_pending_clearance_when_creating_place_with_moderated_tags() -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;

    let new_place = usecases::NewPlace {
        title: "created_place".into(),
        description: "created_place".into(),
        tags: vec![tag.clone()],
        ..default_new_place()
    };
    let created_place = flows::create_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        new_place,
        None,
        None,
    )?;

    assert!(created_place.revision.is_initial());
    assert!(created_place.tags.contains(tag));
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    // Not yet cleared (and invisible)
    assert_eq!(
        None,
        pending_clearances.first().unwrap().last_cleared_revision
    );

    Ok(())
}

#[test]
fn should_deny_creation_of_place_with_moderated_tags_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;

    let new_place = usecases::NewPlace {
        title: "created_place".into(),
        description: "created_place".into(),
        tags: vec![tag.clone()],
        ..default_new_place()
    };
    assert!(flows::create_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        new_place,
        None,
        None,
    )
    .is_err());
    // No pending clearances created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_create_pending_clearance_once_when_updating_place_with_moderated_tags(
) -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;
    let last_cleared_revision = old_place.revision;

    let new_revision = old_place.revision.next();
    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.version = new_revision.into();
    update_place.tags.push(tag.clone());
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision.clone()),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    let mut update_place = usecases::UpdatePlace::from(new_place.clone());
    let new_revision = new_revision.next();
    update_place.version = new_revision.into();
    update_place.tags = vec![];
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;
    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.is_empty());
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    // Pending clearance is unchanged
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    Ok(())
}

#[test]
fn should_deny_adding_of_moderated_tag_to_place_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;

    let new_revision = old_place.revision.next();
    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.version = new_revision.into();
    update_place.tags.push(tag.clone());
    assert!(flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )
    .is_err());
    // No pending clearances created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_deny_removing_of_moderated_tag_from_place_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.confirmed_place;
    assert!(old_place.tags.contains(tag));
    let place_id = &old_place.id;

    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    let new_revision = old_place.revision.next();
    update_place.version = new_revision.into();
    update_place.tags = old_place
        .tags
        .iter()
        .filter(|place_tag| *place_tag != tag)
        .cloned()
        .collect();
    assert!(flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )
    .is_err());
    // No pending clearances created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_create_pending_clearance_when_updating_an_archived_place_with_moderated_tags(
) -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.archived_place;
    let place_id = &fixture.archived_place.id;
    let last_cleared_revision = old_place.revision;

    let new_revision = old_place.revision.next();
    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.version = new_revision.into();
    update_place.tags.push(tag.clone());
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision.clone()),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    Ok(())
}

#[test]
fn should_return_the_last_cleared_revision_when_loading_or_searching_cleared_places(
) -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.confirmed_place;
    assert!(old_place.tags.contains(tag));
    let place_id = &old_place.id;
    let last_cleared_revision = old_place.revision;

    assert_eq!(
        0,
        usecases::clearance::place::count_pending_clearances(
            &*fixture.backend.db_connections.shared()?,
            &org.api_token,
        )?
    );

    let new_title = "new title".to_string();
    assert_ne!(old_place.title, new_title);
    let new_revision = old_place.revision.next();
    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.title = new_title.clone();
    update_place.version = new_revision.into();
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;

    assert_eq!(new_revision, new_place.revision);
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision.clone()),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    // Search uncleared (default)
    let (uncleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec![tag.as_str()],
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, uncleared_search_result.len());
    assert_eq!(new_title, uncleared_search_result.first().unwrap().title);
    // Load uncleared (default)
    let uncleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        None,
    )?;
    assert_eq!(1, uncleared_load_result.len());
    assert_eq!(new_title, uncleared_load_result.first().unwrap().0.title);

    // Search cleared
    let (cleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            org_tag: Some(tag.as_str()),
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, cleared_search_result.len());
    assert_eq!(
        old_place.title,
        cleared_search_result.first().unwrap().title
    );
    // Load cleared
    let cleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        Some(tag.as_str()),
    )?;
    assert_eq!(1, cleared_load_result.len());
    assert_eq!(
        old_place.title,
        cleared_load_result.first().unwrap().0.title
    );

    // Archive, clear, and then confirm (= unarchive) this entry
    flows::review_places(
        &fixture.backend.db_connections,
        &mut *fixture.backend.search_engine.get_mut(),
        &[place_id.as_ref()],
        usecases::Review {
            status: ReviewStatus::Archived,
            context: None,
            comment: None,
            reviewer_email: fixture.user_email.clone(),
        },
    )?;
    assert_eq!(
        1,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );
    assert_eq!(
        1,
        usecases::clearance::place::update_pending_clearances(
            &*fixture.backend.db_connections.exclusive()?,
            &org.api_token,
            &[ClearanceForPlace {
                place_id: place_id.clone(),
                cleared_revision: None,
            }],
        )?
    );
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );
    // Restore archived place by confirming it
    flows::review_places(
        &fixture.backend.db_connections,
        &mut *fixture.backend.search_engine.get_mut(),
        &[place_id.as_ref()],
        usecases::Review {
            status: ReviewStatus::Confirmed,
            context: None,
            comment: None,
            reviewer_email: fixture.user_email.clone(),
        },
    )?;

    // Load & search uncleared (default)
    let uncleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        None,
    )?;
    let (uncleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec![tag.as_str()],
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, uncleared_load_result.len());
    assert_eq!(1, uncleared_search_result.len());
    assert_eq!(new_title, uncleared_load_result.first().unwrap().0.title);
    assert_eq!(new_title, uncleared_search_result.first().unwrap().title);
    assert_eq!(
        ReviewStatus::Confirmed,
        uncleared_load_result.first().unwrap().1
    );
    assert_eq!(
        Some(ReviewStatus::Confirmed),
        uncleared_search_result.first().unwrap().status
    );

    // Load & search cleared - Not filtered, because no more pending clearances
    let cleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        Some(tag.as_str()),
    )?;
    let (cleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            org_tag: Some(tag.as_str()),
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, cleared_load_result.len());
    assert_eq!(1, cleared_search_result.len());
    assert_eq!(new_title, cleared_load_result.first().unwrap().0.title);
    assert_eq!(new_title, cleared_search_result.first().unwrap().title);
    assert_eq!(
        ReviewStatus::Confirmed,
        cleared_load_result.first().unwrap().1
    );
    assert_eq!(
        Some(ReviewStatus::Confirmed),
        cleared_search_result.first().unwrap().status
    );

    Ok(())
}

#[test]
fn should_hide_untagged_cleared_revision_when_loading_or_searching_for_cleared_places(
) -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;
    let last_cleared_revision = old_place.revision;

    let new_title = "new title".to_string();
    assert_ne!(old_place.title, new_title);
    let new_tags = vec![tag.clone()];
    assert_ne!(old_place.tags, new_tags);
    let new_revision = old_place.revision.next();

    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.title = new_title.clone();
    update_place.tags = new_tags.clone();
    update_place.version = new_revision.into();
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision.clone()),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    // Load & search uncleared (default)
    let uncleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        None,
    )?;
    let (uncleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec![tag.as_str()],
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, uncleared_load_result.len());
    assert_eq!(1, uncleared_search_result.len());
    assert_eq!(new_title, uncleared_load_result.first().unwrap().0.title);
    assert_eq!(new_title, uncleared_search_result.first().unwrap().title);
    assert_eq!(new_tags, uncleared_load_result.first().unwrap().0.tags);
    assert_eq!(new_tags, uncleared_search_result.first().unwrap().tags);

    // Load & search cleared
    let cleared_load_result = usecases::load_places(
        &*fixture.backend.db_connections.shared()?,
        &vec![place_id.as_ref()],
        Some(tag.as_str()),
    )?;
    let (cleared_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            org_tag: Some(tag.as_str()),
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    // The cleared initial revision is not tagged and should
    // be removed from the results
    assert!(cleared_load_result.is_empty());
    assert!(cleared_search_result.is_empty());

    Ok(())
}

#[test]
fn should_fail_when_trying_to_clear_future_revisions_of_places() -> flows::Result<()> {
    let mut fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;
    let last_cleared_revision = old_place.revision;

    let new_title = "new title".to_string();
    assert_ne!(old_place.title, new_title);
    let new_tags = vec![tag.clone()];
    assert_ne!(old_place.tags, new_tags);
    let new_revision = old_place.revision.next();

    let mut update_place = usecases::UpdatePlace::from(old_place.clone());
    update_place.title = new_title.clone();
    update_place.tags = new_tags.clone();
    update_place.version = new_revision.into();
    let new_place = flows::update_place(
        &fixture.backend.db_connections,
        fixture.backend.search_engine.get_mut(),
        &fixture.backend.notify,
        place_id.clone(),
        update_place,
        None,
        None,
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_clearances = usecases::clearance::place::list_pending_clearances(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_clearances.len());
    assert_eq!(
        Some(last_cleared_revision.clone()),
        pending_clearances.first().unwrap().last_cleared_revision
    );

    assert_eq!(
        1,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );
    // Try to clear the next, non-existent revision of the place
    assert!(usecases::clearance::place::update_pending_clearances(
        &*fixture.backend.db_connections.exclusive()?,
        &org.api_token,
        &[ClearanceForPlace {
            place_id: place_id.clone(),
            cleared_revision: Some(new_revision.next()),
        }],
    )
    .is_err());
    // Still pending
    assert_eq!(
        1,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_do_nothing_when_clearing_places_without_pending_clearances() -> flows::Result<()> {
    let fixture = PlaceClearanceFixture::new();
    let org = &fixture.organization_with_add_remove_clearance_tag;

    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );
    assert_eq!(
        0,
        usecases::clearance::place::update_pending_clearances(
            &*fixture.backend.db_connections.exclusive()?,
            &org.api_token,
            &[
                ClearanceForPlace {
                    place_id: fixture.created_place.id.clone(),
                    cleared_revision: None,
                },
                ClearanceForPlace {
                    place_id: fixture.confirmed_place.id.clone(),
                    cleared_revision: Some(fixture.confirmed_place.revision),
                }
            ],
        )?
    );
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_clearances_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}
