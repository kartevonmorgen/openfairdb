use crate::core::{prelude::*, usecases};

mod flows {
    pub use super::super::flows::{prelude::*, tests::prelude::BackendFixture, Result};
}

pub struct PlaceAuthorizationFixture {
    backend: flows::BackendFixture,

    user_email: Email,

    // A place without any tags that has been newly created, i.e. initial revision
    created_place: Place,

    // A place without any tags that has been archived
    archived_place: Place,

    // A place with ALL moderated tags that has been confirmed
    confirmed_place: Place,

    // Organization with a moderated tags that allows only add
    // and requires authorization
    organization_with_add_authorized_tag: Organization,

    // Organization with a moderated tags that allows only add
    // and requires authorization
    organization_with_remove_authorized_tag: Organization,

    // Organization with a moderated tags that allows both add
    // and remove and requires authorization
    organization_with_addremove_authorized_tag: Organization,
}

fn default_new_place() -> usecases::NewPlace {
    usecases::NewPlace {
        title: Default::default(),
        description: Default::default(),
        categories: Default::default(),
        email: None,
        telephone: None,
        lat: Default::default(),
        lng: Default::default(),
        street: None,
        zip: None,
        city: None,
        country: None,
        state: None,
        tags: Default::default(),
        homepage: None,
        opening_hours: None,
        license: "CC0-1.0".into(),
        image_url: None,
        image_link_url: None,
    }
}

fn default_search_request<'a>() -> usecases::SearchRequest<'a> {
    usecases::SearchRequest {
        bbox: MapBbox::new(
            MapPoint::from_lat_lng_deg(-90, -180),
            MapPoint::from_lat_lng_deg(90, 180),
        ),
        auth_tag: None,
        categories: vec![],
        hash_tags: vec![],
        ids: vec![],
        status: vec![],
        text: None,
    }
}

impl PlaceAuthorizationFixture {
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

        let confirmed_place = flows::create_place(
            &backend.db_connections,
            &mut *backend.search_engine.borrow_mut(),
            &backend.notify,
            usecases::NewPlace {
                title: "confirmed_place".into(),
                description: "confirmed_place".into(),
                tags: vec![
                    "authadd".into(),
                    "authremove".into(),
                    "authaddremove".into(),
                ],
                ..default_new_place()
            },
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

        // Create organizations with moderated tags
        let organization_without_moderated_tags = Organization {
            id: Id::new(),
            name: "organization_without_moderated_tags".into(),
            api_token: "organization_without_moderated_tags".into(),
            moderated_tags: vec![],
        };
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_without_moderated_tags.clone())
            .unwrap();
        let organization_with_add_authorized_tag = Organization {
            id: Id::new(),
            name: "organization_with_add_authorized_tag".into(),
            api_token: "organization_with_add_authorized_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "authadd".into(),
                moderation_flags: TagModerationFlags::authorize().join(TagModerationFlags::add()),
            }],
        };
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_add_authorized_tag.clone())
            .unwrap();
        let organization_with_remove_authorized_tag = Organization {
            id: Id::new(),
            name: "organization_with_remove_authorized_tag".into(),
            api_token: "organization_with_remove_authorized_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "authremove".into(),
                moderation_flags: TagModerationFlags::authorize()
                    .join(TagModerationFlags::remove()),
            }],
        };
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_remove_authorized_tag.clone())
            .unwrap();
        let organization_with_addremove_authorized_tag = Organization {
            id: Id::new(),
            name: "organization_with_authaddremove_tag".into(),
            api_token: "organization_with_authaddremove_tag".into(),
            moderated_tags: vec![ModeratedTag {
                label: "authaddremove".into(),
                moderation_flags: TagModerationFlags::authorize()
                    .join(TagModerationFlags::add())
                    .join(TagModerationFlags::remove()),
            }],
        };
        backend
            .db_connections
            .exclusive()
            .unwrap()
            .create_org(organization_with_addremove_authorized_tag.clone())
            .unwrap();
        Self {
            backend,
            user_email,
            created_place,
            archived_place,
            confirmed_place,
            organization_with_add_authorized_tag,
            organization_with_remove_authorized_tag,
            organization_with_addremove_authorized_tag,
        }
    }
}

#[test]
fn should_create_pending_authorization_when_creating_place_with_moderated_tags() -> flows::Result<()>
{
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_add_authorized_tag;
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
    )?;

    assert!(created_place.revision.is_initial());
    assert!(created_place.tags.contains(tag));
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_authorizations.len());
    // Not yet authorized (and invisible)
    assert_eq!(
        None,
        pending_authorizations
            .first()
            .unwrap()
            .last_authorized_revision
    );

    Ok(())
}

#[test]
fn should_deny_creation_of_place_with_moderated_tags_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_remove_authorized_tag;
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
    )
    .is_err());
    // No pending authorizations created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_create_pending_authorization_once_when_updating_place_with_moderated_tags(
) -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_addremove_authorized_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;
    let last_authorized_revision = old_place.revision;

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
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_authorizations.len());
    assert_eq!(
        Some(last_authorized_revision.clone()),
        pending_authorizations
            .first()
            .unwrap()
            .last_authorized_revision
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
    )?;
    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.is_empty());
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    // Pending authorization is unchanged
    assert_eq!(1, pending_authorizations.len());
    assert_eq!(
        Some(last_authorized_revision),
        pending_authorizations
            .first()
            .unwrap()
            .last_authorized_revision
    );

    Ok(())
}

#[test]
fn should_deny_adding_of_moderated_tag_to_place_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_remove_authorized_tag;
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
    )
    .is_err());
    // No pending authorizations created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_deny_removing_of_moderated_tag_from_place_if_not_allowed() -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_add_authorized_tag;
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
    )
    .is_err());
    // No pending authorizations created
    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}

#[test]
fn should_create_pending_authorization_when_updating_an_archived_place_with_moderated_tags(
) -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_addremove_authorized_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.archived_place;
    let place_id = &fixture.archived_place.id;
    let last_authorized_revision = old_place.revision;

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
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_authorizations.len());
    assert_eq!(
        Some(last_authorized_revision.clone()),
        pending_authorizations
            .first()
            .unwrap()
            .last_authorized_revision
    );

    Ok(())
}

#[test]
fn should_return_the_last_authorized_revision_when_searching_for_authorized_places(
) -> flows::Result<()> {
    let mut fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_addremove_authorized_tag;
    let tag = &org.moderated_tags.first().unwrap().label;
    let old_place = &fixture.created_place;
    let place_id = &old_place.id;
    let last_authorized_revision = old_place.revision;

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
    )?;

    assert_eq!(new_revision, new_place.revision);
    assert!(new_place.tags.contains(tag));
    let pending_authorizations = usecases::authorization::place::list_pending_authorizations(
        &*fixture.backend.db_connections.shared()?,
        &org.api_token,
        &Default::default(),
    )?;
    assert_eq!(1, pending_authorizations.len());
    assert_eq!(
        Some(last_authorized_revision.clone()),
        pending_authorizations
            .first()
            .unwrap()
            .last_authorized_revision
    );

    // Unauthorized (default)
    let (unauthorized_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec![tag.as_str()],
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, unauthorized_search_result.len());
    assert_eq!(new_title, unauthorized_search_result.first().unwrap().title);
    assert_eq!(new_tags, unauthorized_search_result.first().unwrap().tags);
    // Authorized
    let (authorized_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            auth_tag: Some(tag.as_str()),
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, authorized_search_result.len());
    assert_eq!(
        old_place.title,
        authorized_search_result.first().unwrap().title
    );
    assert_eq!(
        old_place.tags,
        authorized_search_result.first().unwrap().tags
    );

    // Archive, authorize, and then confirm this entry
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
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );
    assert_eq!(
        1,
        usecases::authorization::place::acknowledge_pending_authorizations(
            &*fixture.backend.db_connections.exclusive()?,
            &org.api_token,
            &[AuthorizationForPlace {
                place_id: place_id.clone(),
                authorized_revision: None,
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
            .count_pending_authorizations_for_places(&org.id)
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

    // Unauthorized (default)
    let (unauthorized_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            hash_tags: vec![tag.as_str()],
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, unauthorized_search_result.len());
    assert_eq!(new_title, unauthorized_search_result.first().unwrap().title);
    assert_eq!(
        Some(ReviewStatus::Confirmed),
        unauthorized_search_result.first().unwrap().status
    );
    // Authorized - Not filtered, because no more pending authorizations
    let (authorized_search_result, _) = usecases::search(
        &*fixture.backend.db_connections.shared()?,
        &*fixture.backend.search_engine.borrow(),
        usecases::SearchRequest {
            auth_tag: Some(tag.as_str()),
            ids: vec![place_id.as_ref()],
            ..default_search_request()
        },
        100,
    )?;
    assert_eq!(1, authorized_search_result.len());
    assert_eq!(new_title, authorized_search_result.first().unwrap().title);
    assert_eq!(
        Some(ReviewStatus::Confirmed),
        authorized_search_result.first().unwrap().status
    );

    Ok(())
}

#[test]
fn should_do_nothing_when_acknowledging_places_without_pending_authorizations() -> flows::Result<()>
{
    let fixture = PlaceAuthorizationFixture::new();
    let org = &fixture.organization_with_addremove_authorized_tag;

    assert_eq!(
        0,
        fixture
            .backend
            .db_connections
            .shared()
            .unwrap()
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );
    assert_eq!(
        0,
        usecases::authorization::place::acknowledge_pending_authorizations(
            &*fixture.backend.db_connections.exclusive()?,
            &org.api_token,
            &[
                AuthorizationForPlace {
                    place_id: fixture.created_place.id.clone(),
                    authorized_revision: None,
                },
                AuthorizationForPlace {
                    place_id: fixture.confirmed_place.id.clone(),
                    authorized_revision: None,
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
            .count_pending_authorizations_for_places(&org.id)
            .unwrap()
    );

    Ok(())
}
