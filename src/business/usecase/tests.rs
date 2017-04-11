use super::*;
use business::builder::EntryBuilder;

type RepoResult<T> = result::Result<T, RepoError>;

pub struct MockDb {
    pub entries: Vec<Entry>,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub triples: Vec<Triple>,
    pub users: Vec<User>,
    pub ratings: Vec<Rating>,
    pub comments: Vec<Comment>,
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
            comments: vec![]
        }
    }
}

fn get<T:Clone + Id>(objects: &[T], id: &str) -> RepoResult<T> {
    match objects.iter().find(|x| x.id() == id) {
        Some(x) => Ok(x.clone()),
        None => Err(RepoError::NotFound),
    }
}

fn create<T:Clone + Id>(objects: &mut Vec<T>, e: &T) -> RepoResult<()> {
    if objects.iter().any(|x| x.id() == e.id()) {
        return Err(RepoError::AlreadyExists)
    } else {
        objects.push(e.clone());
    }
    Ok(())
}

fn update<T: Clone + Id>(objects: &mut Vec<T>, e: &T) -> RepoResult<()> {
    if let Some(pos) = objects.iter().position(|x| x.id() == e.id()) {
        objects[pos] = e.clone();
    } else {
        return Err(RepoError::NotFound)
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

    fn get_entry(&self, id: &str) -> RepoResult<Entry> {
        get(&self.entries,id)
    }

    fn get_user(&self, id: &str) -> RepoResult<User> {
        get(&self.users,id)
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

    fn update_entry(&mut self, e: &Entry) -> RepoResult<()> {
        update(&mut self.entries, e)
    }

    fn delete_triple(&mut self, t: &Triple) -> RepoResult<()> {
        self.triples = self.triples.clone().into_iter().filter(|x| x != t).collect();
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
    let now = UTC::now();
    let id = create_new_entry(&mut mock_db, x).unwrap();
    assert!(Uuid::parse_str(&id).is_ok());
    assert_eq!(mock_db.entries.len(),1);
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
fn update_valid_entry(){
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
    let now = UTC::now();
    assert!(update_entry(&mut mock_db, new).is_ok());
    assert_eq!(mock_db.entries.len(),1);
    let x = &mock_db.entries[0];
    assert_eq!(x.street, Some("street".into()));
    assert_eq!(x.description, "bar");
    assert_eq!(x.version, 2);
    assert!(x.created as i64 >= now.timestamp());
    assert!(Uuid::parse_str(&x.id).is_ok());
}

#[test]
fn update_entry_with_invalid_version(){
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
                RepoError::NotFound => { },
                _ => {
                    panic!("invalid error type");
                }
            }
        },
        _ => {
            panic!("invalid error type");
        }
    }
    assert_eq!(mock_db.entries.len(),0);
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
    assert_eq!(mock_db.tags.len(),2);
    assert_eq!(mock_db.entries.len(),1);
    assert_eq!(mock_db.triples.len(),2);
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
fn update_valid_entry_with_tags(){
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
    assert_eq!(res,vec!["bio".to_string(),"fair".to_string()])
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
    assert_eq!(res,vec!["foo".to_string(),"bar".to_string()])
}

#[test]
fn create_user_with_invalid_name(){
    let mut db = MockDb::new();
    let u = NewUser{
        username: "".into(),
        password: "bar".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "also&invalid".into(),
        password: "bar".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "thisisvalid".into(),
        password: "very_secret".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_ok());
}

#[test]
fn create_user_with_invalid_password(){
    let mut db = MockDb::new();
    let u = NewUser{
        username: "user".into(),
        password: "".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "user".into(),
        password: "not valid".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "user".into(),
        password: "validpass".into(),
        email: "foo@baz.io".into()
    };
    assert!(create_new_user(&mut db,u).is_ok());
}

#[test]
fn create_user_with_invalid_email(){
    let mut db = MockDb::new();
    let u = NewUser{
        username: "user".into(),
        password: "pass".into(),
        email: "".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "user".into(),
        password: "pass".into(),
        email: "fooo@".into()
    };
    assert!(create_new_user(&mut db,u).is_err());
    let u = NewUser{
        username: "user".into(),
        password: "pass".into(),
        email: "fooo@bar.io".into()
    };
    assert!(create_new_user(&mut db,u).is_ok());
}

#[test]
fn encrypt_user_password(){
    let mut db = MockDb::new();
    let u = NewUser{
        username: "user".into(),
        password: "pass".into(),
        email: "foo@bar.io".into()
    };
    assert!(create_new_user(&mut db,u).is_ok());
    assert!(db.users[0].password != "pass");
    assert!(bcrypt::verify("pass", &db.users[0].password));
}


#[test]
fn rate_non_existing_entry(){
    let mut db = MockDb::new();
    assert!(rate_entry(&mut db,RateEntry{
        entry: "does_not_exist".into(),
        comment: "a comment".into(),
        context: RatingContext::Fair,
        user: None,
        value: 2
    }).is_err());
}

#[test]
fn rate_with_empty_comment(){
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "".into(),
        context: RatingContext::Fair,
        user: None,
        value: 2
    }).is_err());
}

#[test]
fn rate_with_invalid_value_comment(){
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "comment".into(),
        context: RatingContext::Fair,
        user: None,
        value: 3
    }).is_err());
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "comment".into(),
        context: RatingContext::Fair,
        user: None,
        value: -2
    }).is_err());
}

#[test]
fn rate_without_login(){
    let mut db = MockDb::new();
    let e = Entry::build().id("foo").finish();
    db.entries = vec![e];
    assert!(rate_entry(&mut db,RateEntry{
        entry: "foo".into(),
        comment: "comment".into(),
        context: RatingContext::Fair,
        user: None,
        value: 2
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
