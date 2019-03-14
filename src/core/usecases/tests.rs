use super::*;

use crate::core::{
    usecases,
    util::{
        filter::{self, InBBox},
        geo,
    },
};

use failure::{format_err, Fallible};
use std::{cell::RefCell, result};

//TODO: move tests to corresponding usecase

type RepoResult<T> = result::Result<T, RepoError>;

trait Id {
    fn id(&self) -> &str;
}

impl Id for Entry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Event {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Category {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Tag {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for User {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Comment {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Rating {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for BboxSubscription {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Organization {
    fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Default)]
pub struct MockDb {
    pub entries: RefCell<Vec<Entry>>,
    pub events: RefCell<Vec<Event>>,
    pub categories: Vec<Category>,
    pub tags: RefCell<Vec<Tag>>,
    pub users: RefCell<Vec<User>>,
    pub ratings: RefCell<Vec<Rating>>,
    pub comments: RefCell<Vec<Comment>>,
    pub bbox_subscriptions: Vec<BboxSubscription>,
    pub orgs: Vec<Organization>,
}

impl MockDb {
    pub fn get_entries_by_bbox(&self, bbox: &geo::MapBbox) -> RepoResult<Vec<Entry>> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .filter(|e| e.in_bbox(bbox))
            .cloned()
            .collect())
    }
}

impl EntryIndexer for MockDb {
    fn add_or_update_entry(&mut self, entry: &Entry, _ratings: &AvgRatings) -> Fallible<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Ok(entry), self.db.get_entry(&entry.id).as_ref());
        debug_assert!(entry == &self.get_entry(&entry.id).unwrap());
        Ok(())
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        // Nothing to do, the entry has already been stored
        // in the database.
        //debug_assert_eq!(Err(RepoError::NotFound), self.db.get_entry(&id));
        debug_assert!(self.get_entry(&id).is_err());
        Ok(())
    }

    fn flush(&mut self) -> Fallible<()> {
        Ok(())
    }
}

impl EntryIndex for MockDb {
    fn query_entries(&self, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<IndexedEntry>> {
        let mut entries = if let Some(ref bbox) = query.bbox {
            self.get_entries_by_bbox(&bbox)
        } else {
            self.all_entries()
        }
        .map_err(|err| format_err!("{}", err))?;

        if !query.ids.is_empty() {
            entries = entries
                .into_iter()
                .filter(|e| query.ids.iter().any(|id| &e.id == id))
                .collect();
        }

        if !query.categories.is_empty() {
            entries = entries
                .into_iter()
                .filter(filter::entries_by_category_ids(&query.categories))
                .collect();
        }

        let indexed_entries = entries
            .into_iter()
            .take(limit)
            .filter(&*filter::entries_by_tags_or_search_text(
                query.text.as_ref().map(String::as_str).unwrap_or(""),
                &query.tags,
            ))
            .map(|e| IndexedEntry {
                id: e.id,
                pos: e.location.pos,
                title: e.title,
                description: e.description,
                categories: e.categories,
                tags: e.tags,
                ratings: Default::default(),
            })
            /*id: e.id,
            title: e.title,
            description: e.description,
            categories: e.categories,*/
            .collect();

        Ok(indexed_entries)
    }
}

fn get<T: Clone + Id>(objects: &[T], id: &str) -> RepoResult<T> {
    match objects.iter().find(|x| x.id() == id) {
        Some(x) => Ok(x.clone()),
        None => Err(RepoError::NotFound),
    }
}

fn create<T: Clone + Id>(objects: &mut Vec<T>, e: T) -> RepoResult<()> {
    if objects.iter().any(|x| x.id() == e.id()) {
        return Err(RepoError::AlreadyExists);
    } else {
        objects.push(e);
    }
    Ok(())
}

fn update<T: Clone + Id>(objects: &mut Vec<T>, e: &T) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.id() == e.id()) {
        objects[pos] = e.clone();
    } else {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

fn delete<T: Clone + Id>(objects: &mut Vec<T>, id: &str) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.id() == id) {
        objects.remove(pos);
    } else {
        return Err(RepoError::NotFound);
    }
    Ok(())
}

impl EntryGateway for MockDb {
    fn create_entry(&self, e: Entry) -> RepoResult<()> {
        create(&mut self.entries.borrow_mut(), e)
    }
    fn get_entry(&self, id: &str) -> RepoResult<Entry> {
        get(&self.entries.borrow(), id).and_then(|e| {
            if e.archived.is_none() {
                Ok(e)
            } else {
                Err(RepoError::NotFound)
            }
        })
    }
    fn get_entries(&self, ids: &[&str]) -> RepoResult<Vec<Entry>> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .filter(|e| e.archived.is_none() && ids.iter().any(|id| &e.id == id))
            .cloned()
            .collect())
    }
    fn all_entries(&self) -> RepoResult<Vec<Entry>> {
        Ok(self
            .entries
            .borrow()
            .iter()
            .filter(|e| e.archived.is_none())
            .cloned()
            .collect())
    }
    fn count_entries(&self) -> RepoResult<usize> {
        self.all_entries().map(|v| v.len())
    }

    fn update_entry(&self, e: &Entry) -> RepoResult<()> {
        update(&mut self.entries.borrow_mut(), e)
    }

    fn import_multiple_entries(&mut self, _entries: &[Entry]) -> RepoResult<()> {
        unimplemented!();
    }

    fn archive_entries(&self, _ids: &[&str], _archived: Timestamp) -> RepoResult<usize> {
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

    fn all_events(&self) -> RepoResult<Vec<Event>> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|e| e.archived.is_none())
            .cloned()
            .collect())
    }

    fn count_events(&self) -> RepoResult<usize> {
        self.all_events().map(|v| v.len())
    }

    fn update_event(&self, e: &Event) -> RepoResult<()> {
        update(&mut self.events.borrow_mut(), e)
    }

    fn archive_events(&self, _ids: &[&str], _archived: Timestamp) -> RepoResult<usize> {
        unimplemented!();
    }

    fn delete_event(&self, id: &str) -> RepoResult<()> {
        delete(&mut self.events.borrow_mut(), id)
    }
}

impl UserGateway for MockDb {
    fn create_user(&self, u: User) -> RepoResult<()> {
        create(&mut self.users.borrow_mut(), u)
    }

    fn get_user(&self, username: &str) -> RepoResult<User> {
        let users: &Vec<User> = &self
            .users
            .borrow()
            .iter()
            .filter(|u| u.username == username)
            .cloned()
            .collect();
        if users.len() > 0 {
            Ok(users[0].clone())
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn get_users_by_email(&self, email: &str) -> RepoResult<Vec<User>> {
        let users: Vec<_> = self
            .users
            .borrow()
            .iter()
            .filter(|u| u.email == email)
            .cloned()
            .collect();
        if users.len() > 0 {
            Ok(users)
        } else {
            Err(RepoError::NotFound)
        }
    }

    fn all_users(&self) -> RepoResult<Vec<User>> {
        Ok(self.users.borrow().clone())
    }

    fn count_users(&self) -> RepoResult<usize> {
        self.all_users().map(|v| v.len())
    }

    fn delete_user(&self, u_id: &str) -> RepoResult<()> {
        self.users.borrow_mut().retain(|u| u.id != u_id);
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
            if c.archived.is_none() {
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
            .filter(|c| ids.iter().any(|id| &c.id == id) && c.archived.is_none())
            .cloned()
            .collect())
    }

    fn load_comments_of_rating(&self, rating_id: &str) -> RepoResult<Vec<Comment>> {
        Ok(self
            .comments
            .borrow()
            .iter()
            .filter(|c| &c.rating_id == rating_id && c.archived.is_none())
            .cloned()
            .collect())
    }

    fn archive_comments(&self, _ids: &[&str], _archived: Timestamp) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_comments_of_ratings(
        &self,
        _rating_ids: &[&str],
        _archived: Timestamp,
    ) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_comments_of_entries(
        &self,
        _entry_ids: &[&str],
        _archived: Timestamp,
    ) -> RepoResult<usize> {
        unimplemented!();
    }
}

impl OrganizationGateway for MockDb {
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
    fn get_all_tags_owned_by_orgs(&self) -> RepoResult<Vec<String>> {
        Ok(self
            .orgs
            .iter()
            .flat_map(|o| o.owned_tags.clone())
            .collect())
    }
}

impl RatingRepository for MockDb {
    fn create_rating(&self, r: Rating) -> RepoResult<()> {
        create(&mut self.ratings.borrow_mut(), r)
    }

    fn load_rating(&self, id: &str) -> RepoResult<Rating> {
        get(&self.ratings.borrow(), id).and_then(|r| {
            if r.archived.is_none() {
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
            .filter(|r| ids.iter().any(|id| &r.id == id) && r.archived.is_none())
            .cloned()
            .collect())
    }

    fn load_ratings_of_entry(&self, entry_id: &str) -> RepoResult<Vec<Rating>> {
        Ok(self
            .ratings
            .borrow()
            .iter()
            .filter(|r| r.archived.is_none() && &r.entry_id == entry_id)
            .cloned()
            .collect())
    }

    fn load_entry_ids_of_ratings(&self, _ids: &[&str]) -> RepoResult<Vec<String>> {
        unimplemented!();
    }
    fn archive_ratings(&self, _ids: &[&str], _archived: Timestamp) -> RepoResult<usize> {
        unimplemented!();
    }
    fn archive_ratings_of_entries(
        &self,
        _entry_ids: &[&str],
        _archived: Timestamp,
    ) -> RepoResult<usize> {
        unimplemented!();
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

    fn create_category_if_it_does_not_exist(&mut self, e: &Category) -> RepoResult<()> {
        if let Err(err) = create(&mut self.categories, e.clone()) {
            match err {
                RepoError::AlreadyExists => {
                    // that's ok
                }
                _ => return Err(err),
            }
        }
        Ok(())
    }

    fn create_bbox_subscription(&mut self, s: &BboxSubscription) -> RepoResult<()> {
        create(&mut self.bbox_subscriptions, s.clone())
    }

    fn all_categories(&self) -> RepoResult<Vec<Category>> {
        Ok(self.categories.clone())
    }

    fn all_tags(&self) -> RepoResult<Vec<Tag>> {
        Ok(self.tags.borrow().clone())
    }
    fn count_tags(&self) -> RepoResult<usize> {
        self.all_tags().map(|v| v.len())
    }

    fn all_bbox_subscriptions(&self) -> RepoResult<Vec<BboxSubscription>> {
        Ok(self.bbox_subscriptions.clone())
    }

    fn delete_bbox_subscription(&mut self, s_id: &str) -> RepoResult<()> {
        self.bbox_subscriptions = self
            .bbox_subscriptions
            .iter()
            .filter(|s| s.id != s_id)
            .cloned()
            .collect();
        Ok(())
    }
}

mod tests {
    use super::*;
    use chrono::prelude::*;
    #[test]
    fn receive_different_user() {
        let db = MockDb::default();
        db.users.borrow_mut().push(User {
            id: "1".into(),
            username: "a".into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "a@foo.bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        });
        db.users.borrow_mut().push(User {
            id: "2".into(),
            username: "b".into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "b@foo.bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        });
        assert!(get_user(&db, "a", "b").is_err());
        assert!(get_user(&db, "a", "a").is_ok());
    }

    #[test]
    fn create_bbox_subscription() {
        let mut db = MockDb::default();
        let bbox_new = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(-71.3, 179.5),
            MapPoint::from_lat_lng_deg(88.2, -160),
        );

        let username = "a";
        assert!(db
            .create_user(User {
                id: "123".into(),
                username: username.into(),
                password: "secret".parse::<Password>().unwrap(),
                email: "abc@abc.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());
        assert!(usecases::subscribe_to_bbox(bbox_new, username.into(), &mut db,).is_ok());

        let bbox_subscription = db.all_bbox_subscriptions().unwrap()[0].clone();
        assert_eq!(
            bbox_subscription.bbox.north_east().lat(),
            LatCoord::from_deg(88.2)
        );
    }

    #[test]
    fn modify_bbox_subscription() {
        let mut db = MockDb::default();

        let bbox_old = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(49.0, 5.0),
            MapPoint::from_lat_lng_deg(50.0, 10.0),
        );

        let bbox_new = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(9.0, 5.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );

        let username = "a";
        assert!(db
            .create_user(User {
                id: "123".into(),
                username: username.into(),
                password: "secret".parse::<Password>().unwrap(),
                email: "abc@abc.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());

        let bbox_subscription = BboxSubscription {
            id: "123".into(),
            bbox: bbox_old,
            username: "a".into(),
        };
        db.create_bbox_subscription(&bbox_subscription.clone())
            .unwrap();

        usecases::subscribe_to_bbox(bbox_new, username.into(), &mut db).unwrap();

        let bbox_subscriptions: Vec<_> = db
            .all_bbox_subscriptions()
            .unwrap()
            .into_iter()
            .filter(|s| &*s.username == "a")
            .collect();

        assert_eq!(bbox_subscriptions.len(), 1);
        assert_eq!(
            bbox_subscriptions[0].clone().bbox.north_east().lat(),
            LatCoord::from_deg(10.0)
        );
    }

    #[test]
    fn get_bbox_subscriptions() {
        let mut db = MockDb::default();

        let bbox1 = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(49.0, 5.0),
            MapPoint::from_lat_lng_deg(50.0, 10.0),
        );

        let bbox2 = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(9.0, 5.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );

        let user1 = "a";
        assert!(db
            .create_user(User {
                id: user1.into(),
                username: user1.into(),
                password: "secret1".parse::<Password>().unwrap(),
                email: "abc@abc.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());
        let bbox_subscription = BboxSubscription {
            id: "1".into(),
            bbox: bbox1,
            username: "a".into(),
        };
        assert!(db
            .create_bbox_subscription(&bbox_subscription.clone())
            .is_ok());

        let user2 = "b";
        assert!(db
            .create_user(User {
                id: user2.into(),
                username: user2.into(),
                password: "secret2".parse::<Password>().unwrap(),
                email: "abc@abc.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());
        let bbox_subscription2 = BboxSubscription {
            id: "2".into(),
            bbox: bbox2,
            username: "b".into(),
        };
        assert!(db
            .create_bbox_subscription(&bbox_subscription2.clone())
            .is_ok());
        let bbox_subscriptions = usecases::get_bbox_subscriptions(user2.into(), &mut db);
        assert!(bbox_subscriptions.is_ok());
        assert_eq!(bbox_subscriptions.unwrap()[0].id, "2");
    }

    #[test]
    fn email_addresses_by_coordinate() {
        let mut db = MockDb::default();
        let bbox_new = geo::MapBbox::new(
            MapPoint::from_lat_lng_deg(0.0, 0.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );

        let username = "a";
        let u_id = "123".to_string();
        db.create_user(User {
            id: u_id.clone(),
            username: username.into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .unwrap();

        usecases::subscribe_to_bbox(bbox_new, username, &mut db).unwrap();

        let email_addresses =
            usecases::email_addresses_by_coordinate(&db, MapPoint::from_lat_lng_deg(5.0, 5.0))
                .unwrap();
        assert_eq!(email_addresses.len(), 1);
        assert_eq!(email_addresses[0], "abc@abc.de");

        let no_email_addresses =
            usecases::email_addresses_by_coordinate(&db, MapPoint::from_lat_lng_deg(20.0, 20.0))
                .unwrap();
        assert_eq!(no_email_addresses.len(), 0);
    }

    #[test]
    fn delete_user() {
        let mut db = MockDb::default();
        let username = "a".to_string();
        let u_id = "1".to_string();
        assert!(db
            .create_user(User {
                id: u_id.clone(),
                username: username.clone(),
                password: "secret".parse::<Password>().unwrap(),
                email: "abc@abc.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());
        let username = "b".to_string();
        let u_id = "2".to_string();
        assert!(db
            .create_user(User {
                id: u_id.clone(),
                username: username.clone(),
                password: "secret".parse::<Password>().unwrap(),
                email: "abcd@abcd.de".into(),
                email_confirmed: true,
                role: Role::Guest,
            })
            .is_ok());
        assert_eq!(db.count_users().unwrap(), 2);

        assert!(usecases::delete_user(&mut db, "1", "1").is_ok());
        assert_eq!(db.count_users().unwrap(), 1);
    }

    #[test]
    fn receive_event_with_creators_email() {
        let mut db = MockDb::default();
        db.create_user(User {
            id: "x".into(),
            username: "user".into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "abc@abc.de".into(),
            email_confirmed: true,
            role: Role::Guest,
        })
        .unwrap();
        db.create_event(Event {
            id: "x".into(),
            title: "t".into(),
            description: None,
            start: NaiveDateTime::from_timestamp(0, 0),
            end: None,
            contact: None,
            location: None,
            homepage: None,
            tags: vec![],
            created_by: Some("user".into()),
            registration: None,
            organizer: None,
            archived: None,
        })
        .unwrap();
        let e = usecases::get_event(&mut db, "x").unwrap();
        assert_eq!(e.created_by.unwrap(), "abc@abc.de");
    }

    #[test]
    fn tag_lists() {
        assert_eq!(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            prepare_tag_list(vec!["  a  c #b ".to_string()])
        );
    }
}
