use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Db;
use super::filter;
use super::validate::Validate;
use uuid::Uuid;
use std::collections::HashMap;

type Result<T> = result::Result<T,Error>;

trait Id {
    fn id(&self) -> String;
}

impl Id for Entry {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Category {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Tag {
    fn id(&self) -> String {
        self.id.clone()
    }
}

fn triple_id(t: &Triple) -> String {
    let (s_type, s_id) = match t.subject {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id)
    };
    let (o_type, o_id) = match t.object {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id)
    };
    let p_type = match t.predicate {
        Relation::IsTaggedWith => "is_tagged_with"
    };
    format!("{}-{}-{}-{}-{}",s_type,s_id,p_type,o_type,o_id)
}

impl Id for Triple {
    fn id(&self) -> String {
        triple_id(self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewEntry {
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
    license     : String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    id          : String,
    version     : u64,
    title       : String,
    description : String,
    lat         : f64,
    lng         : f64,
    street      : Option<String>,
    zip         : Option<String>,
    city        : Option<String>,
    country     : Option<String>,
    email       : Option<String>,
    telephone   : Option<String>,
    homepage    : Option<String>,
    categories  : Vec<String>,
    tags        : Vec<String>,
}

fn create_missing_tags<D:Db>(db: &mut D, tags: &Vec<String>) -> Result<()> {
    let existing_tags = db.all_tags()?;
    for new_t in tags {
        if !existing_tags.iter().any(|t|t.id == *new_t){
            db.create_tag(&Tag{id:new_t.clone()})?;
        }
    }
    Ok(())
}

struct Diff<T> {
    new: Vec<T>,
    deleted: Vec<T>
}

fn get_triple_diff(old: &Vec<Triple>, new: &Vec<Triple>) -> Diff<Triple> {

    let to_create = new
        .iter()
        .filter(|t|!old.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    let to_delete = old
        .iter()
        .filter(|t|!new.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    Diff{
        new: to_create,
        deleted: to_delete
    }
}


fn set_tag_relations<D:Db>(db: &mut D, entry: &str, tags: &Vec<String>) -> Result<()> {
    create_missing_tags(db, tags)?;
    let subject = ObjectId::Entry(entry.into());
    let old_triples = db.all_triples()?
        .into_iter()
        .filter(|x|x.subject == subject)
        .filter(|x|x.predicate == Relation::IsTaggedWith)
        .collect::<Vec<Triple>>();
    let new_triples = tags
        .into_iter()
        .map(|x| Triple{
            subject: subject.clone(),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag(x.clone())
        })
        .collect::<Vec<Triple>>();

    let diff = get_triple_diff(&old_triples, &new_triples);

    for t in diff.new {
        db.create_triple(&t)?;
    }
    for t in diff.deleted {
        db.delete_triple(&t)?;
    }
    Ok(())
}

pub fn get_tag_ids<D:Db>(db: &D) -> Result<Vec<String>> {
    let mut tags : Vec<String> = db
        .all_triples()?
        .into_iter()
        .filter(|t|t.predicate == Relation::IsTaggedWith)
        .filter_map(|t| match t.object {
           ObjectId::Tag(id) => Some(id),
            _ => None
        })
        .collect();
    tags.dedup();
    Ok(tags)
}

pub fn get_tag_ids_for_entry_id(triples: &Vec<Triple>, entry_id : &str) -> Vec<Tag> {
    triples
        .iter()
        .filter(&*filter::triple_by_entry_id(entry_id))
        .filter(|triple| triple.predicate == Relation::IsTaggedWith)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::Tag(ref tag_id) => Some(tag_id),
                _ => None
            })
        .cloned()
        .map(|tag_id|Tag{id: tag_id})
        .collect()
}

pub fn get_tags_by_entry_ids<D:Db>(db : &D, ids : &Vec<String>) -> Result<HashMap<String, Vec<Tag>>> {
    let triples = db.all_triples()?;
    Ok(ids
        .iter()
        .map(|id|(
            id.clone(),
            get_tag_ids_for_entry_id(&triples, id)
        ))
        .collect())
}

pub fn get_entries<D:Db>(db : &D, ids : &Vec<String>) -> Result<Vec<Entry>> {
    let entries = db
        .all_entries()?
        .into_iter()
        .filter(|e|ids.iter().any(|id| *id == e.id))
        .collect();
    Ok(entries)
}

pub fn create_new_entry<D: Db>(db: &mut D, e: NewEntry) -> Result<String>
 {
    let new_entry = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  UTC::now().timestamp() as u64,
        version     :  0,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  Some(e.license)
    };
    new_entry.validate()?;
    db.create_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(new_entry.id)
}

pub fn update_entry<D: Db>(db: &mut D, e: UpdateEntry) -> Result<()> {
    let old : Entry = db.get_entry(&e.id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let new_entry = Entry{
        id          :  e.id,
        created     :  UTC::now().timestamp() as u64,
        version     :  e.version,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  old.license
    };
    db.update_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use super::*;

    type RepoResult<T> = result::Result<T, RepoError>;

    pub struct MockDb {
        pub entries: Vec<Entry>,
        pub categories: Vec<Category>,
        pub tags: Vec<Tag>,
        pub triples: Vec<Triple>,
    }

    impl MockDb {
        pub fn new() -> MockDb {
            MockDb {
                entries: vec![],
                categories: vec![],
                tags: vec![],
                triples: vec![]
            }
        }
    }

    fn get<T:Clone + Id>(objects: &Vec<T>, id: &str) -> RepoResult<T> {
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

        fn get_entry(&self, id: &str) -> RepoResult<Entry> {
            get(&self.entries,id)
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
    assert_eq!(res,vec![Tag{id:"bio".into()},Tag{id:"fair".into()}])
}
