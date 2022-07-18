use std::{cell::RefCell, result};

use anyhow::Result as Fallible;

use super::*;
use crate::core::{usecases, util::geo};

//TODO: move tests to corresponding usecase

type RepoResult<T> = result::Result<T, RepoError>;

trait Key {
    fn key(&self) -> &str;
}

impl Key for (Place, ReviewStatus) {
    fn key(&self) -> &str {
        self.0.id.as_ref()
    }
}

impl Key for Event {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

impl Key for Category {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

impl Key for Tag {
    fn key(&self) -> &str {
        &self.id
    }
}

impl Key for User {
    fn key(&self) -> &str {
        &self.email
    }
}

impl Key for Comment {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

impl Key for Rating {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

impl Key for BboxSubscription {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

impl Key for Organization {
    fn key(&self) -> &str {
        self.id.as_ref()
    }
}

#[derive(Default)]
pub struct MockDb {
    pub entries: RefCell<Vec<(Place, ReviewStatus)>>,
    pub events: RefCell<Vec<Event>>,
    pub tags: RefCell<Vec<Tag>>,
    pub users: RefCell<Vec<User>>,
    pub ratings: RefCell<Vec<Rating>>,
    pub comments: RefCell<Vec<Comment>>,
    pub bbox_subscriptions: RefCell<Vec<BboxSubscription>>,
    pub orgs: Vec<Organization>,
    pub token: RefCell<Vec<UserToken>>,
}

impl UserTokenRepo for MockDb {
    fn replace_user_token(&self, token: UserToken) -> RepoResult<EmailNonce> {
        for x in &mut self.token.borrow_mut().iter_mut() {
            if x.email_nonce.email == token.email_nonce.email {
                *x = token.clone();
                return Ok(token.email_nonce);
            }
        }
        self.token.borrow_mut().push(token.clone());
        Ok(token.email_nonce)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> RepoResult<UserToken> {
        if let Some(index) = self.token.borrow().iter().enumerate().find_map(|(i, x)| {
            if x.email_nonce.email == email_nonce.email && x.email_nonce.nonce == email_nonce.nonce
            {
                Some(i)
            } else {
                None
            }
        }) {
            Ok(self.token.borrow_mut().swap_remove(index))
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> RepoResult<usize> {
        let len_before = self.token.borrow().len();
        self.token
            .borrow_mut()
            .retain(|x| x.expires_at >= expired_before);
        let len_after = self.token.borrow().len();
        debug_assert!(len_before >= len_after);
        Ok(len_before - len_after)
    }

    fn get_user_token_by_email(&self, _email: &str) -> RepoResult<UserToken> {
        unimplemented!()
    }
}

pub struct DummySearchEngine;

impl Indexer for DummySearchEngine {
    fn flush_index(&mut self) -> Fallible<()> {
        Ok(())
    }
}

impl IdIndex for DummySearchEngine {
    fn query_ids(
        &self,
        _mode: IndexQueryMode,
        _query: &IndexQuery,
        _limit: usize,
    ) -> Fallible<Vec<Id>> {
        unimplemented!();
    }
}

impl IdIndexer for DummySearchEngine {
    fn remove_by_id(&self, _id: &Id) -> Fallible<()> {
        Ok(())
    }
}

impl PlaceIndex for DummySearchEngine {
    fn query_places(&self, _query: &IndexQuery, _limit: usize) -> Fallible<Vec<IndexedPlace>> {
        unimplemented!();
    }
}

impl PlaceIndexer for DummySearchEngine {
    fn add_or_update_place(
        &self,
        _place: &Place,
        _status: ReviewStatus,
        _ratings: &AvgRatings,
    ) -> Fallible<()> {
        Ok(())
    }
}

impl EventIndexer for DummySearchEngine {
    fn add_or_update_event(&self, _event: &Event) -> Fallible<()> {
        Ok(())
    }
}

impl EventAndPlaceIndexer for DummySearchEngine {}

fn get<T: Clone + Key>(objects: &[T], id: &str) -> RepoResult<T> {
    match objects.iter().find(|x| x.key() == id) {
        Some(x) => Ok(x.clone()),
        None => Err(RepoError::NotFound),
    }
}

fn create<T: Clone + Key>(objects: &mut Vec<T>, e: T) -> RepoResult<()> {
    if objects.iter().any(|x| x.key() == e.key()) {
        return Err(RepoError::AlreadyExists);
    } else {
        objects.push(e);
    }
    Ok(())
}

fn create_or_replace<T: Clone + Key>(objects: &mut Vec<T>, e: T) -> RepoResult<()> {
    for elem in objects.iter_mut() {
        if elem.key() == e.key() {
            *elem = e;
            return Ok(());
        }
    }
    objects.push(e);
    Ok(())
}

fn update<T: Clone + Key>(objects: &mut [T], e: &T) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.key() == e.key()) {
        objects[pos] = e.clone();
    } else {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

impl PlaceRepo for MockDb {
    fn create_or_update_place(&self, place: Place) -> RepoResult<()> {
        create_or_replace(
            &mut self.entries.borrow_mut(),
            (place, ReviewStatus::Created),
        )
    }
    fn get_place(&self, id: &str) -> RepoResult<(Place, ReviewStatus)> {
        get(&self.entries.borrow(), id).and_then(|(p, s)| {
            if s != ReviewStatus::Archived {
                Ok((p, s))
            } else {
                Err(RepoError::NotFound)
            }
        })
    }
    fn get_places(&self, ids: &[&str]) -> RepoResult<Vec<(Place, ReviewStatus)>> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .filter(|(p, s)| {
                *s != ReviewStatus::Archived && ids.iter().any(|id| p.id.as_str() == *id)
            })
            .cloned()
            .collect())
    }
    fn all_places(&self) -> RepoResult<Vec<(Place, ReviewStatus)>> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .filter(|(_, s)| *s != ReviewStatus::Archived)
            .cloned()
            .collect())
    }
    fn recently_changed_places(
        &self,
        _params: &RecentlyChangedEntriesParams,
        _pagination: &Pagination,
    ) -> RepoResult<Vec<(Place, ReviewStatus, ActivityLog)>> {
        unimplemented!();
    }
    fn most_popular_place_revision_tags(
        &self,
        _params: &MostPopularTagsParams,
        _pagination: &Pagination,
    ) -> RepoResult<Vec<TagFrequency>> {
        unimplemented!();
    }
    fn count_places(&self) -> RepoResult<usize> {
        self.all_places().map(|v| v.len())
    }

    fn review_places(
        &self,
        _ids: &[&str],
        _status: ReviewStatus,
        _activity: &ActivityLog,
    ) -> RepoResult<usize> {
        unimplemented!();
    }

    fn get_place_history(
        &self,
        _id: &str,
        _revision: Option<Revision>,
    ) -> RepoResult<PlaceHistory> {
        unimplemented!();
    }

    fn load_place_revision(&self, _id: &str, _rev: Revision) -> RepoResult<(Place, ReviewStatus)> {
        unimplemented!();
    }
}

impl EventGateway for MockDb {
    fn create_event(&self, e: Event) -> RepoResult<()> {
        create(&mut self.events.borrow_mut(), e)
    }

    fn get_event(&self, id: &str) -> RepoResult<Event> {
        get(&self.events.borrow(), id).and_then(|e| {
            if e.archived.is_none() {
                Ok(e)
            } else {
                Err(RepoError::NotFound)
            }
        })
    }

    fn all_events_chronologically(&self) -> RepoResult<Vec<Event>> {
        let mut events: Vec<_> = self
            .events
            .borrow()
            .iter()
            .filter(|e| e.archived.is_none())
            .cloned()
            .collect();
        events.sort_by(|a, b| a.start.cmp(&b.start));
        Ok(events)
    }

    fn get_events_chronologically(&self, ids: &[&str]) -> RepoResult<Vec<Event>> {
        let mut events: Vec<_> = self
            .events
            .borrow()
            .iter()
            .filter(|e| ids.iter().any(|id| e.id.as_str() == *id))
            .filter(|e| e.archived.is_none())
            .cloned()
            .collect();
        events.sort_by(|a, b| a.start.cmp(&b.start));
        Ok(events)
    }

    fn count_events(&self) -> RepoResult<usize> {
        self.all_events_chronologically().map(|v| v.len())
    }

    fn update_event(&self, e: &Event) -> RepoResult<()> {
        update(&mut self.events.borrow_mut(), e)
    }

    fn archive_events(&self, _ids: &[&str], _archived: Timestamp) -> RepoResult<usize> {
        unimplemented!();
    }

    fn delete_event_with_matching_tags(&self, _id: &str, _tags: &[&str]) -> RepoResult<bool> {
        unimplemented!();
    }

    fn is_event_owned_by_any_organization(&self, _id: &str) -> RepoResult<bool> {
        unimplemented!();
    }
}

impl UserGateway for MockDb {
    fn create_user(&self, u: &User) -> RepoResult<()> {
        create(&mut self.users.borrow_mut(), u.clone())
    }

    fn try_get_user_by_email(&self, email: &str) -> RepoResult<Option<User>> {
        Ok(self
            .users
            .borrow()
            .iter()
            .find(|u| u.email == email)
            .cloned())
    }

    fn get_user_by_email(&self, email: &str) -> RepoResult<User> {
        self.try_get_user_by_email(email)?
            .ok_or(RepoError::NotFound)
    }

    fn all_users(&self) -> RepoResult<Vec<User>> {
        Ok(self.users.borrow().clone())
    }

    fn count_users(&self) -> RepoResult<usize> {
        self.all_users().map(|v| v.len())
    }

    fn delete_user_by_email(&self, email: &str) -> RepoResult<()> {
        self.users.borrow_mut().retain(|u| u.email != email);
        Ok(())
    }

    fn update_user(&self, u: &User) -> RepoResult<()> {
        update(&mut self.users.borrow_mut(), u)
    }
}

impl CommentRepository for MockDb {
    fn create_comment(&self, c: Comment) -> RepoResult<()> {
        create(&mut self.comments.borrow_mut(), c)
    }

    fn load_comment(&self, id: &str) -> RepoResult<Comment> {
        get(&self.comments.borrow(), id).and_then(|c| {
            if c.archived_at.is_none() {
                Ok(c)
            } else {
                Err(RepoError::NotFound)
            }
        })
    }

    fn load_comments(&self, ids: &[&str]) -> RepoResult<Vec<Comment>> {
        Ok(self
            .comments
            .borrow()
            .iter()
            .filter(|c| ids.iter().any(|id| c.id.as_str() == *id) && c.archived_at.is_none())
            .cloned()
            .collect())
    }

    fn load_comments_of_rating(&self, rating_id: &str) -> RepoResult<Vec<Comment>> {
        Ok(self
            .comments
            .borrow()
            .iter()
            .filter(|c| c.rating_id.as_str() == rating_id && c.archived_at.is_none())
            .cloned()
            .collect())
    }

    fn archive_comments(&self, _ids: &[&str], _activity: &Activity) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_comments_of_ratings(
        &self,
        _rating_ids: &[&str],
        _activity: &Activity,
    ) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_comments_of_places(
        &self,
        _place_ids: &[&str],
        _activity: &Activity,
    ) -> RepoResult<usize> {
        unimplemented!();
    }
}

impl OrganizationRepo for MockDb {
    fn create_org(&mut self, o: Organization) -> RepoResult<()> {
        create(&mut self.orgs, o)
    }
    fn get_org_by_api_token(&self, token: &str) -> RepoResult<Organization> {
        let o = self
            .orgs
            .iter()
            .find(|o| o.api_token == token)
            .ok_or(RepoError::NotFound)?;
        Ok(o.clone())
    }
    fn map_tag_to_clearance_org_id(&self, tag: &str) -> RepoResult<Option<Id>> {
        Ok(self
            .orgs
            .iter()
            .find(|o| {
                o.moderated_tags
                    .iter()
                    .any(|mod_tag| mod_tag.require_clearance && mod_tag.label == tag)
            })
            .map(|o| o.id.clone()))
    }
    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> RepoResult<Vec<(Id, ModeratedTag)>> {
        Ok(self
            .orgs
            .iter()
            .filter(|o| Some(&o.id) != excluded_org_id)
            .flat_map(|o| {
                o.moderated_tags
                    .clone()
                    .into_iter()
                    .map(move |t| (o.id.clone(), t))
            })
            .collect())
    }
}

impl RatingRepository for MockDb {
    fn create_rating(&self, r: Rating) -> RepoResult<()> {
        create(&mut self.ratings.borrow_mut(), r)
    }

    fn load_rating(&self, id: &str) -> RepoResult<Rating> {
        get(&self.ratings.borrow(), id).and_then(|r| {
            if r.archived_at.is_none() {
                Ok(r)
            } else {
                Err(RepoError::NotFound)
            }
        })
    }

    fn load_ratings(&self, ids: &[&str]) -> RepoResult<Vec<Rating>> {
        Ok(self
            .ratings
            .borrow()
            .iter()
            .filter(|r| ids.iter().any(|id| r.id.as_str() == *id) && r.archived_at.is_none())
            .cloned()
            .collect())
    }

    fn load_ratings_of_place(&self, place_id: &str) -> RepoResult<Vec<Rating>> {
        Ok(self
            .ratings
            .borrow()
            .iter()
            .filter(|r| r.archived_at.is_none() && r.place_id.as_str() == place_id)
            .cloned()
            .collect())
    }

    fn load_place_ids_of_ratings(&self, _ids: &[&str]) -> RepoResult<Vec<String>> {
        unimplemented!();
    }
    fn archive_ratings(&self, _ids: &[&str], _activity: &Activity) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_ratings_of_places(
        &self,
        _place_ids: &[&str],
        _activity: &Activity,
    ) -> RepoResult<usize> {
        unimplemented!();
    }
}

impl PlaceClearanceRepo for MockDb {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        _pending_clearance: &PendingClearanceForPlace,
    ) -> RepoResult<usize> {
        Ok(org_ids.len())
    }

    fn count_pending_clearances_for_places(&self, _org_id: &Id) -> RepoResult<u64> {
        Ok(0)
    }

    fn list_pending_clearances_for_places(
        &self,
        _org_id: &Id,
        _pagination: &Pagination,
    ) -> RepoResult<Vec<PendingClearanceForPlace>> {
        Ok(vec![])
    }

    fn load_pending_clearances_for_places(
        &self,
        _org_id: &Id,
        _place_ids: &[&str],
    ) -> RepoResult<Vec<PendingClearanceForPlace>> {
        Ok(vec![])
    }

    fn update_pending_clearances_for_places(
        &self,
        _org_id: &Id,
        _clearances: &[ClearanceForPlace],
    ) -> RepoResult<usize> {
        Ok(0)
    }

    fn cleanup_pending_clearances_for_places(&self, _org_id: &Id) -> RepoResult<u64> {
        Ok(0)
    }
}

impl Db for MockDb {
    fn create_tag_if_it_does_not_exist(&self, e: &Tag) -> RepoResult<()> {
        if let Err(err) = create(&mut self.tags.borrow_mut(), e.clone()) {
            match err {
                RepoError::AlreadyExists => {
                    // that's ok
                }
                _ => return Err(err),
            }
        }
        Ok(())
    }

    fn create_bbox_subscription(&self, s: &BboxSubscription) -> RepoResult<()> {
        create(&mut self.bbox_subscriptions.borrow_mut(), s.clone())
    }

    fn all_tags(&self) -> RepoResult<Vec<Tag>> {
        Ok(self.tags.borrow().clone())
    }
    fn count_tags(&self) -> RepoResult<usize> {
        self.all_tags().map(|v| v.len())
    }

    fn all_bbox_subscriptions(&self) -> RepoResult<Vec<BboxSubscription>> {
        Ok(self.bbox_subscriptions.borrow().clone())
    }

    fn all_bbox_subscriptions_by_email(
        &self,
        user_email: &str,
    ) -> RepoResult<Vec<BboxSubscription>> {
        Ok(self
            .bbox_subscriptions
            .borrow()
            .iter()
            .filter(|s| s.user_email == user_email)
            .cloned()
            .collect())
    }

    fn delete_bbox_subscriptions_by_email(&self, user_email: &str) -> RepoResult<()> {
        self.bbox_subscriptions
            .borrow_mut()
            .retain(|s| s.user_email != user_email);
        Ok(())
    }
}

#[test]
fn receive_different_user() {
    let db = MockDb::default();
    db.users.borrow_mut().push(User {
        email: "a@foo.bar".into(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    });
    db.users.borrow_mut().push(User {
        email: "b@foo.bar".into(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    });
    assert!(get_user(&db, "a@foo.bar", "b@foo.bar").is_err());
    assert!(get_user(&db, "a@foo.bar", "a@foo.bar").is_ok());
}

#[test]
fn create_bbox_subscription() {
    let db = MockDb::default();
    let bbox_new = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(-71.3, 179.5),
        MapPoint::from_lat_lng_deg(88.2, -160),
    );

    assert!(db
        .create_user(&User {
            email: "abc@abc.de".into(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());
    assert!(usecases::subscribe_to_bbox(&db, "abc@abc.de".into(), bbox_new).is_ok());

    let bbox_subscription = db.all_bbox_subscriptions().unwrap()[0].clone();
    assert_eq!(
        bbox_subscription.bbox.northeast().lat(),
        LatCoord::from_deg(88.2)
    );
}

#[test]
fn modify_bbox_subscription() {
    let db = MockDb::default();

    let bbox_old = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(49.0, 5.0),
        MapPoint::from_lat_lng_deg(50.0, 10.0),
    );

    let bbox_new = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(9.0, 5.0),
        MapPoint::from_lat_lng_deg(10.0, 10.0),
    );

    assert!(db
        .create_user(&User {
            email: "abc@abc.de".into(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());

    let bbox_subscription = BboxSubscription {
        id: "123".into(),
        user_email: "abc@abc.de".into(),
        bbox: bbox_old,
    };
    db.create_bbox_subscription(&bbox_subscription).unwrap();

    usecases::subscribe_to_bbox(&db, "abc@abc.de".into(), bbox_new).unwrap();

    let bbox_subscriptions: Vec<_> = db
        .all_bbox_subscriptions()
        .unwrap()
        .into_iter()
        .filter(|s| &*s.user_email == "abc@abc.de")
        .collect();

    assert_eq!(bbox_subscriptions.len(), 1);
    assert_eq!(
        bbox_subscriptions[0].clone().bbox.northeast().lat(),
        LatCoord::from_deg(10.0)
    );
}

#[test]
fn get_bbox_subscriptions() {
    let db = MockDb::default();

    let bbox1 = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(49.0, 5.0),
        MapPoint::from_lat_lng_deg(50.0, 10.0),
    );

    let bbox2 = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(9.0, 5.0),
        MapPoint::from_lat_lng_deg(10.0, 10.0),
    );

    assert!(db
        .create_user(&User {
            email: "a@abc.de".into(),
            email_confirmed: true,
            password: "secret1".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());
    let bbox_subscription = BboxSubscription {
        id: "1".into(),
        user_email: "a@abc.de".into(),
        bbox: bbox1,
    };
    assert!(db.create_bbox_subscription(&bbox_subscription).is_ok());

    assert!(db
        .create_user(&User {
            email: "b@abc.de".into(),
            email_confirmed: true,
            password: "secret2".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());
    let bbox_subscription2 = BboxSubscription {
        id: "2".into(),
        user_email: "b@abc.de".into(),
        bbox: bbox2,
    };
    assert!(db.create_bbox_subscription(&bbox_subscription2).is_ok());
    let bbox_subscriptions = usecases::get_bbox_subscriptions(&db, "b@abc.de");
    assert!(bbox_subscriptions.is_ok());
    assert_eq!(bbox_subscriptions.unwrap()[0].id, "2".into());
}

#[test]
fn email_addresses_by_coordinate() {
    let db = MockDb::default();
    let bbox_new = geo::MapBbox::new(
        MapPoint::from_lat_lng_deg(0.0, 0.0),
        MapPoint::from_lat_lng_deg(10.0, 10.0),
    );

    db.create_user(&User {
        email: "abc@abc.de".into(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    })
    .unwrap();

    usecases::subscribe_to_bbox(&db, "abc@abc.de".into(), bbox_new).unwrap();

    let email_addresses =
        usecases::email_addresses_by_coordinate(&db, MapPoint::from_lat_lng_deg(5.0, 5.0)).unwrap();
    assert_eq!(email_addresses.len(), 1);
    assert_eq!(email_addresses[0], "abc@abc.de");

    let no_email_addresses =
        usecases::email_addresses_by_coordinate(&db, MapPoint::from_lat_lng_deg(20.0, 20.0))
            .unwrap();
    assert_eq!(no_email_addresses.len(), 0);
}

#[test]
fn delete_user() {
    let db = MockDb::default();
    assert!(db
        .create_user(&User {
            email: "abc@abc.de".into(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());
    assert!(db
        .create_user(&User {
            email: "abcd@abcd.de".into(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        })
        .is_ok());
    assert_eq!(db.count_users().unwrap(), 2);

    assert!(usecases::delete_user(&db, "abc@abc.de", "abc@abc.de").is_ok());
    assert_eq!(db.count_users().unwrap(), 1);
}

#[test]
fn receive_event_with_creators_email() {
    let db = MockDb::default();
    db.create_user(&User {
        email: "abc@abc.de".into(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    })
    .unwrap();
    db.create_event(Event {
        id: "x".into(),
        title: "t".into(),
        description: None,
        start: Timestamp::from_seconds(0),
        end: None,
        contact: None,
        location: None,
        homepage: None,
        tags: vec![],
        created_by: Some("abc@abc.de".into()),
        registration: None,
        archived: None,
        image_url: None,
        image_link_url: None,
    })
    .unwrap();
    let e = usecases::get_event(&db, "x").unwrap();
    assert_eq!(e.created_by.unwrap(), "abc@abc.de");
}

#[test]
fn tag_lists() {
    assert_eq!(
        vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e-f".to_string()
        ],
        prepare_tag_list(vec!["  A\n#d\tc #B ", "#", "#e-f"].into_iter())
    );
}
