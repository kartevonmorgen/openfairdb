use super::*;
use business::builder::EntryBuilder;
use entities;
use business;
use uuid::Uuid;

type RepoResult<T> = result::Result<T, RepoError>;

pub struct MockDb {
    pub entries: Vec<Entry>,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub triples: Vec<Triple>,
    pub users: Vec<User>,
    pub ratings: Vec<Rating>,
    pub comments: Vec<Comment>,
    pub bbox_subscriptions: Vec<BboxSubscription>
}

impl MockDb {
    pub fn new() -> MockDb {
        MockDb {
            entries: vec![],
            categories: vec![],
            tags: vec![],
            triples: vec![],
            users: vec![],
            ratings: vec![],
            comments: vec![],
            bbox_subscriptions: vec![]
        }
    }
}

fn get<T: Clone + Id>(objects: &[T], id: &str) -> RepoResult<T> {
    match objects.iter().find(|x| x.id() == id) {
        Some(x) => Ok(x.clone()),
        None => Err(RepoError::NotFound),
    }
}

fn create<T: Clone + Id>(objects: &mut Vec<T>, e: &T) -> RepoResult<()> {
    if objects.iter().any(|x| x.id() == e.id()) {
        return Err(RepoError::AlreadyExists);
    } else {
        objects.push(e.clone());
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

impl Db for MockDb {
    fn create_entry(&mut self, e: &Entry) -> RepoResult<()> {
        create(&mut self.entries, e)
    }

    fn create_tag(&mut self, e: &Tag) -> RepoResult<()> {
        create(&mut self.tags, e)
    }

    fn create_triple(&mut self, e: &Triple) -> RepoResult<()> {
        create(&mut self.triples, e)
    }

    fn create_user(&mut self, u: &User) -> RepoResult<()> {
        create(&mut self.users, u)
    }

    fn create_comment(&mut self, c: &Comment) -> RepoResult<()> {
        create(&mut self.comments, c)
    }

    fn create_rating(&mut self, r: &Rating) -> RepoResult<()> {
        create(&mut self.ratings, r)
    }

    fn create_bbox_subscription(&mut self, s: &BboxSubscription) -> RepoResult<()> {
        create(&mut self.bbox_subscriptions, s)
    }

    fn get_entry(&self, id: &str) -> RepoResult<Entry> {
        get(&self.entries, id)
    }

    fn get_user(&self, username: &str) -> RepoResult<User> {
        let users : &Vec<User> = &self.users
            .iter()
            .filter(|u| u.username == username)
            .cloned()
            .collect();
        if users.len() > 0 {
            Ok(users[0].clone())
        } else{
            Err(RepoError::NotFound)
        }
    }

    fn all_entries(&self) -> RepoResult<Vec<Entry>> {
        Ok(self.entries.clone())
    }

    fn all_categories(&self) -> RepoResult<Vec<Category>> {
        Ok(self.categories.clone())
    }

    fn all_tags(&self) -> RepoResult<Vec<Tag>> {
        Ok(self.tags.clone())
    }

    fn all_triples(&self) -> RepoResult<Vec<Triple>> {
        Ok(self.triples.clone())
    }

    fn all_ratings(&self) -> RepoResult<Vec<Rating>> {
        Ok(self.ratings.clone())
    }

    fn all_comments(&self) -> RepoResult<Vec<Comment>> {
        Ok(self.comments.clone())
    }

    fn all_users(&self) -> RepoResult<Vec<User>> {
        Ok(self.users.clone())
    }

    fn all_bbox_subscriptions(&self) -> RepoResult<Vec<BboxSubscription>> {
        Ok(self.bbox_subscriptions.clone())
    }

    fn update_entry(&mut self, e: &Entry) -> RepoResult<()> {
        update(&mut self.entries, e)
    }

    fn confirm_email_address(&mut self, u_id: &str) -> RepoResult<User> {
        let a : String = self.all_users()?[0].clone().id;
        let b : String = u_id.to_string();
        println!("u.id: {:?}", a);
        println!("u_id: {:?}", b);

        let users : Vec<User> = self.all_users()?
            .into_iter()
            .filter(|u| u.id == u_id.to_string())
            .collect();
        println!("filtered users: {:?}", users);
        if users.len() > 0 {
            let mut u = users[0].clone();
            println!("user: {:?}", u);
            u.email_confirmed = true;
            update(&mut self.users, &u)?;
            Ok(u)
        } else{
            Err(RepoError::NotFound)
        }
    }    

    fn delete_triple(&mut self, t: &Triple) -> RepoResult<()> {
        self.triples = self.triples
            .clone()
            .into_iter()
            .filter(|x| x != t)
            .collect();
        Ok(())
    }

    fn delete_bbox_subscription(&mut self, s_id: &str) -> RepoResult<()>{
        self.bbox_subscriptions = vec![];
        self.triples = self.triples
            .clone()
            .into_iter()
            .filter(|t| t.object != ObjectId::BboxSubscription(s_id.into()))
            .collect();
        Ok(())
    }

    fn delete_user(&mut self, u_id: &str) -> RepoResult<()>{
        self.users = self.users
            .clone()
            .into_iter()
            .filter(|u| u.id != u_id)
            .collect();
        Ok(())
    }
}

#[test]
fn create_new_valid_entry() {
    let x = NewEntry {
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec![],
        license     : "CC0-1.0".into()
    };
    let mut mock_db = MockDb::new();
    let now = Utc::now();
    let id = create_new_entry(&mut mock_db, x).unwrap();
    assert!(Uuid::parse_str(&id).is_ok());
    assert_eq!(mock_db.entries.len(), 1);
    let x = &mock_db.entries[0];
    assert_eq!(x.title, "foo");
    assert_eq!(x.description, "bar");
    assert_eq!(x.version, 0);
    assert!(x.created as i64 >= now.timestamp());
    assert!(Uuid::parse_str(&x.id).is_ok());
    assert_eq!(x.id, id);
}

#[test]
fn create_entry_with_invalid_email() {
    let x = NewEntry {
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : Some("fooo-not-ok".into()),
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec![],
        license     : "CC0-1.0".into()
    };
    let mut mock_db: MockDb = MockDb::new();
    assert!(create_new_entry(&mut mock_db, x).is_err());
}

#[test]
fn update_valid_entry() {
    let id = Uuid::new_v4().simple().to_string();
    let old = Entry {
        id          : id.clone(),
        version     : 1,
        created     : 0,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        license     : None
    };
    let new = UpdateEntry {
        id          : id.clone(),
        version     : 2,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : Some("street".into()),
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec![],
    };
    let mut mock_db = MockDb::new();
    mock_db.entries = vec![old];
    let now = Utc::now();
    assert!(update_entry(&mut mock_db, new).is_ok());
    assert_eq!(mock_db.entries.len(), 1);
    let x = &mock_db.entries[0];
    assert_eq!(x.street, Some("street".into()));
    assert_eq!(x.description, "bar");
    assert_eq!(x.version, 2);
    assert!(x.created as i64 >= now.timestamp());
    assert!(Uuid::parse_str(&x.id).is_ok());
}

#[test]
fn update_entry_with_invalid_version() {
    let id = Uuid::new_v4().simple().to_string();
    let old = Entry {
        id          : id.clone(),
        version     : 3,
        created     : 0,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        license     : None
    };
    let new = UpdateEntry {
        id          : id.clone(),
        version     : 3,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : Some("street".into()),
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec![],
    };
    let mut mock_db = MockDb::new();
    mock_db.entries = vec![old];
    let result = update_entry(&mut mock_db, new);
    assert!(result.is_err());
    match result.err().unwrap() {
        Error::Repo(err) => {
            match err {
                RepoError::InvalidVersion => { },
                _ => {
                    panic!("invalid error type");
                }
            }
        },
        _ => {
            panic!("invalid error type");
        }
    }
    assert_eq!(mock_db.entries.len(),1);
}

#[test]
fn update_non_existing_entry(){
    let id = Uuid::new_v4().simple().to_string();
    let new = UpdateEntry {
        id          : id.clone(),
        version     : 4,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : Some("street".into()),
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec![],
    };
    let mut mock_db = MockDb::new();
    mock_db.entries = vec![];
    let result = update_entry(&mut mock_db, new);
    assert!(result.is_err());
    match result.err().unwrap() {
        Error::Repo(err) => {
            match err {
                RepoError::NotFound => {}
                _ => {
                    panic!("invalid error type");
                }
            }
        }
        _ => {
            panic!("invalid error type");
        }
    }
    assert_eq!(mock_db.entries.len(), 0);
}

#[test]
fn add_new_valid_entry_with_tags() {
    let x = NewEntry {
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec!["foo".into(),"bar".into()],
        license     : "CC0-1.0".into()
    };
    let mut mock_db = MockDb::new();
    create_new_entry(&mut mock_db, x).unwrap();
    assert_eq!(mock_db.tags.len(), 2);
    assert_eq!(mock_db.entries.len(), 1);
    assert_eq!(mock_db.triples.len(), 2);
}

#[test]
fn calc_triple_diff(){
    let old = vec![
        Triple{
            subject: ObjectId::Entry("foo".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("bio".into())
        },
        Triple{
            subject: ObjectId::Entry("foo".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("fair".into())
        },
        Triple{
            subject: ObjectId::Entry("bar".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("unknown".into())
        }];
    let new = vec![
        Triple{
            subject: ObjectId::Entry("foo".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("vegan".into())
        },
        Triple{
            subject: ObjectId::Entry("foo".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("bio".into())
        },
        Triple{
            subject: ObjectId::Entry("bar".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("unknown".into())
        },
        Triple{
            subject: ObjectId::Entry("bar".into()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("new".into())
        }];
    let diff = get_triple_diff(&old,&new);

    assert_eq!(diff.new.len(),2);
    assert_eq!(diff.new[0].object,ObjectId::Tag("vegan".into()));
    assert_eq!(diff.new[1].object,ObjectId::Tag("new".into()));
    assert_eq!(diff.deleted.len(),1);
    assert_eq!(diff.deleted[0].object,ObjectId::Tag("fair".into()));
}

#[test]
fn update_valid_entry_with_tags() {
    let id = Uuid::new_v4().simple().to_string();
    let old = Entry {
        id          : id.clone(),
        version     : 1,
        created     : 0,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : None,
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        license     : None
    };
    let new = UpdateEntry {
        id          : id.clone(),
        version     : 2,
        title       : "foo".into(),
        description : "bar".into(),
        lat         : 0.0,
        lng         : 0.0,
        street      : Some("street".into()),
        zip         : None,
        city        : None,
        country     : None,
        email       : None,
        telephone   : None,
        homepage    : None,
        categories  : vec![],
        tags        : vec!["vegan".into()],
    };
    let mut mock_db = MockDb::new();
    mock_db.entries = vec![old];
    mock_db.triples = vec![
        Triple{
            subject: ObjectId::Entry(id.clone()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("bio".into()),
        },
        Triple{
            subject: ObjectId::Entry(id.clone()),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag("fair".into()),
        }
    ];
    let res = get_tags_by_entry_ids(&mock_db, &vec![id.clone()]).unwrap();
    assert_eq!(res.get(&id).cloned().unwrap(), vec![Tag{id: "bio".into()},Tag{id:"fair".into()}]);
    assert!(update_entry(&mut mock_db, new).is_ok());
    let res = get_tags_by_entry_ids(&mock_db, &vec![id.clone()]).unwrap();
    assert_eq!(res.get(&id).cloned().unwrap(), vec![Tag{id: "vegan".into()}]);
}

#[test]
fn get_correct_tag_ids_for_entry_id() {
    let triples = vec![
            Triple{
                subject: ObjectId::Entry("a".into()),
                predicate: Relation::IsTaggedWith,
                object: ObjectId::Tag("bio".into()),
            },
            Triple{
                subject: ObjectId::Entry("a".into()),
                predicate: Relation::IsTaggedWith,
                object: ObjectId::Tag("fair".into()),
            },
            Triple{
                subject: ObjectId::Entry("b".into()),
                predicate: Relation::IsTaggedWith,
                object: ObjectId::Tag("fair".into()),
            }
        ];
    let res = get_tag_ids_for_entry_id(&triples, "a");
    assert_eq!(res, vec!["bio".to_string(), "fair".to_string()])
}

#[test]
fn get_correct_rating_ids_for_entry_id() {
    let triples = vec![
            Triple{
                subject: ObjectId::Entry("a".into()),
                predicate: Relation::IsRatedWith,
                object: ObjectId::Rating("foo".into()),
            },
            Triple{
                subject: ObjectId::Entry("a".into()),
                predicate: Relation::IsRatedWith,
                object: ObjectId::Rating("bar".into()),
            },
            Triple{
                subject: ObjectId::Entry("a".into()),
                predicate: Relation::IsTaggedWith,
                object: ObjectId::Tag("bio".into()),
            },
            Triple{
                subject: ObjectId::Entry("b".into()),
                predicate: Relation::IsRatedWith,
                object: ObjectId::Rating("baz".into()),
            }
        ];
    let res = get_rating_ids_for_entry_id(&triples, "a");
    assert_eq!(res, vec!["foo".to_string(), "bar".to_string()])
}

#[test]
fn create_user_with_invalid_name() {
    let mut db = MockDb::new();
    let u = NewUser {
        username: "".into(),
        password: "bar".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "also&invalid".into(),
        password: "bar".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "thisisvalid".into(),
        password: "very_secret".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_ok());
}

#[test]
fn create_user_with_invalid_password() {
    let mut db = MockDb::new();
    let u = NewUser {
        username: "user".into(),
        password: "".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "user".into(),
        password: "not valid".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "user".into(),
        password: "validpass".into(),
        email: "foo@baz.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_ok());
}

#[test]
fn create_user_with_invalid_email() {
    let mut db = MockDb::new();
    let u = NewUser {
        username: "user".into(),
        password: "pass".into(),
        email: "".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "user".into(),
        password: "pass".into(),
        email: "fooo@".into(),
    };
    assert!(create_new_user(&mut db, u).is_err());
    let u = NewUser {
        username: "user".into(),
        password: "pass".into(),
        email: "fooo@bar.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_ok());
}

#[test]
fn create_user_with_existing_username(){
    let mut db = MockDb::new();
    db.users = vec![User{
        id: "123".into(),
        username: "foo".into(),
        password: "bar".into(),
        email: "baz@foo.bar".into(),
        email_confirmed: true
    }];
    let u = NewUser{
        username: "foo".into(),
        password: "pass".into(),
        email: "user@server.tld".into(),
    };
    match create_new_user(&mut db,u).err().unwrap() {
        Error::Parameter(err) => match err {
            ParameterError::UserExists => {
                // ok
            },
            _ => { panic!("invalid error") }
        },
        _ => { panic!("invalid error") }
    }
}

#[test]
fn email_unconfirmed_on_default(){
    let mut db = MockDb::new();
    let u = NewUser {
        username: "user".into(),
        password: "pass".into(),
        email: "foo@bar.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_ok());
    assert_eq!(db.users[0].email_confirmed, false);
}

#[test]
fn encrypt_user_password(){
    let mut db = MockDb::new();
    let u = NewUser {
        username: "user".into(),
        password: "pass".into(),
        email: "foo@bar.io".into(),
    };
    assert!(create_new_user(&mut db, u).is_ok());
    assert!(db.users[0].password != "pass");
    assert!(bcrypt::verify("pass", &db.users[0].password));
}

#[test]
fn rate_non_existing_entry() {
    let mut db = MockDb::new();
    assert!(rate_entry(&mut db,RateEntry{
        entry: "does_not_exist".into(),
        title: "title".into(),
        comment: "a comment".into(),
        context: RatingContext::Fairness,
        user: None,
        value: 2,
        source: Some("source".into())
    }).is_err());
}

#[test]
fn rate_with_empty_comment() {
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "".into(),
        title: "title".into(),
        context: RatingContext::Fairness,
        user: None,
        value: 2,
        source: Some("source".into())
    }).is_err());
}

#[test]
fn rate_with_invalid_value_comment() {
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "comment".into(),
        title: "title".into(),
        context: RatingContext::Fairness,
        user: None,
        value: 3,
        source: Some("source".into())
    }).is_err());
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        title: "title".into(),
        comment: "comment".into(),
        context: RatingContext::Fairness,
        user: None,
        value: -2,
        source: Some("source".into())
    }).is_err());
}

#[test]
fn rate_without_login() {
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "comment".into(),
        title: "title".into(),
        context: RatingContext::Fairness,
        user: None,
        value: 2,
        source: Some("source".into())
    }).is_ok());
    assert_eq!(db.ratings.len(),1);
    assert_eq!(db.comments.len(),1);
    assert_eq!(db.triples.len(),2);
    assert_eq!(db.triples[0].subject,ObjectId::Entry("foo".into()));
    assert_eq!(db.triples[0].predicate,Relation::IsRatedWith);
    assert!(match db.triples[0].object {
        ObjectId::Rating(_) => true, _ => false
    });
    assert!(match db.triples[1].subject {
        ObjectId::Rating(_) => true, _ => false
    });
    assert_eq!(db.triples[1].predicate,Relation::IsCommentedWith);
    assert!(match db.triples[1].object {
        ObjectId::Comment(_) => true, _ => false
    });
}

#[test]
fn receive_different_user() {
    let mut db = MockDb::new();
    db.users = vec![
        User{
            id: "1".into(),
            username: "a".into(),
            password: "a".into(),
            email: "a@foo.bar".into(),
            email_confirmed: true
        },
        User{
            id: "2".into(),
            username: "b".into(),
            password: "b".into(),
            email: "b@foo.bar".into(),
            email_confirmed: true
        }];
    assert!(get_user(&mut db, "1", "b").is_err());
    assert!(get_user(&mut db, "1", "a").is_ok());
}

#[test]
fn create_bbox_subscription(){
    let mut db = MockDb::new();
    let bbox_new = entities::Bbox{
        north_east: Coordinate{
            lat: 10.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 10.0,
            lng: 5.0
        }
    };

    let username = "a";
    assert!(db.create_user(&User{
        id: "123".into(),
        username: username.into(),
        password: username.into(),
        email: "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());
    assert!(business::usecase::create_or_modify_subscription(&bbox_new, username.into(), &mut db).is_ok());

    let bbox_subscription = db.all_bbox_subscriptions().unwrap()[0].clone();
    assert_eq!(bbox_subscription.north_east_lat, 10.0);   
}

#[test]
fn modify_bbox_subscription(){
    let mut db = MockDb::new();

    let bbox_old = entities::Bbox{
        north_east: Coordinate{
            lat: 50.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 50.0,
            lng: 5.0
        }
    };

    let bbox_new = entities::Bbox{
        north_east: Coordinate{
            lat: 10.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 10.0,
            lng: 5.0
        }
    };

    let username = "a";
    assert!(db.create_user(&User{
        id: "123".into(),
        username: username.into(),
        password: username.into(),
        email: "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());

    let bbox_subscription = BboxSubscription {
        id: "123".into(),
        north_east_lat: bbox_old.north_east.lat,
        north_east_lng: bbox_old.north_east.lng,
        south_west_lat: bbox_old.south_west.lat,
        south_west_lng: bbox_old.south_west.lng
    };
    assert!(db.create_bbox_subscription(&bbox_subscription.clone()).is_ok());

    assert!(db.create_triple(
        &Triple{
            subject: ObjectId::User("a".into()),
            predicate: Relation::SubscribedTo,
            object: ObjectId::BboxSubscription("123".into()),
    }).is_ok());

    assert!(business::usecase::create_or_modify_subscription(&bbox_new, username.into(), &mut db).is_ok());
    
    let user_subscriptions : Vec<String>  = db.triples.clone()
        .into_iter()
        .filter_map(|triple| match triple {
            Triple {
                subject     : ObjectId::User(ref u_id),
                predicate   : Relation::SubscribedTo,
                object      : ObjectId::BboxSubscription(ref s_id)
            } => Some((u_id.clone(), s_id.clone())),
            _ => None
        })
        .filter(|user_subscription| *user_subscription.0 == *username)
        .map(|user_and_subscription| user_and_subscription.1)
        .collect();

    let bbox_subscriptions : Vec<BboxSubscription> = db.bbox_subscriptions
        .into_iter()
        .filter(|subscription| subscription.id == user_subscriptions[0])
        .collect();
    assert_eq!(bbox_subscriptions.len(), 1);
    assert_eq!(bbox_subscriptions[0].clone().north_east_lat, 10.0);
}


#[test]
fn get_bbox_subscriptions(){
    let mut db = MockDb::new();

    let bbox1 = entities::Bbox{
        north_east: Coordinate{
            lat: 50.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 50.0,
            lng: 5.0
        }
    };

    let bbox2 = entities::Bbox{
        north_east: Coordinate{
            lat: 10.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 10.0,
            lng: 5.0
        }
    };

    let user1 = "a";
    assert!(db.create_user(&User{
        id:         user1.into(),
        username:   user1.into(),
        password:   user1.into(),
        email:      "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());
    let bbox_subscription = BboxSubscription {
        id: "1".into(),
        north_east_lat: bbox1.north_east.lat,
        north_east_lng: bbox1.north_east.lng,
        south_west_lat: bbox1.south_west.lat,
        south_west_lng: bbox1.south_west.lng
    };
    assert!(db.create_bbox_subscription(&bbox_subscription.clone()).is_ok());
    assert!(db.create_triple(
        &Triple{
            subject: ObjectId::User("a".into()),
            predicate: Relation::SubscribedTo,
            object: ObjectId::BboxSubscription("1".into()),
    }).is_ok());

    let user2 = "b";
    assert!(db.create_user(&User{
        id:         user2.into(),
        username:   user2.into(),
        password:   user2.into(),
        email:      "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());
    let bbox_subscription2 = BboxSubscription {
        id: "2".into(),
        north_east_lat: bbox2.north_east.lat,
        north_east_lng: bbox2.north_east.lng,
        south_west_lat: bbox2.south_west.lat,
        south_west_lng: bbox2.south_west.lng
    };
    assert!(db.create_bbox_subscription(&bbox_subscription2.clone()).is_ok());
    assert!(db.create_triple(
        &Triple{
            subject: ObjectId::User("b".into()),
            predicate: Relation::SubscribedTo,
            object: ObjectId::BboxSubscription("2".into()),
    }).is_ok());

    let bbox_subscriptions = business::usecase::get_bbox_subscriptions(user2.into(), &mut db);
    assert!(bbox_subscriptions.is_ok());
    assert_eq!(bbox_subscriptions.unwrap()[0].id, "2");
}

#[test]
fn email_addresses_to_notify(){
    let mut db = MockDb::new();
    let bbox_new = entities::Bbox{
        north_east: Coordinate{
            lat: 10.0,
            lng: 10.0
        },
        south_west: Coordinate{
            lat: 0.0,
            lng: 0.0
        }
    };

    let username = "a".to_string();
    let u_id = "123".to_string();
    assert!(db.create_user(&User{
        id: u_id.clone(),
        username: username.clone(),
        password: username,
        email: "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());

    assert!(business::usecase::create_or_modify_subscription(
        &bbox_new, u_id, &mut db).is_ok());
    
    let email_addresses = business::usecase::email_addresses_to_notify(&5.0, &5.0, &mut db);
    assert_eq!(email_addresses.len(), 1);
    assert_eq!(email_addresses[0], "abc@abc.de");

    let no_email_addresses = business::usecase::email_addresses_to_notify(&20.0, &20.0, &mut db);
    assert_eq!(no_email_addresses.len(), 0);
}

#[test]
fn delete_user(){
    let mut db = MockDb::new();
    let username = "a".to_string();
    let u_id = "1".to_string();
    assert!(db.create_user(&User{
        id: u_id.clone(),
        username: username.clone(),
        password: username,
        email: "abc@abc.de".into(),
        email_confirmed: true
    }).is_ok());
    let username = "b".to_string();
    let u_id = "2".to_string();
    assert!(db.create_user(&User{
        id: u_id.clone(),
        username: username.clone(),
        password: username,
        email: "abcd@abcd.de".into(),
        email_confirmed: true
    }).is_ok());
    assert_eq!(db.users.len(), 2);

    assert!(business::usecase::delete_user(&mut db, "1", "1").is_ok());
    assert_eq!(db.users.len(), 1);
}
