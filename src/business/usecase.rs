use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Db;
use super::validate::Validate;
use uuid::Uuid;

type Result<T> = result::Result<T,Error>;

trait Id {
    fn id(&self) -> &str;
}

impl Id for Entry {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Id for Category {
    fn id(&self) -> &str {
        &self.id
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
}

pub fn create_new_entry<D: Db>(db: &mut D, e: NewEntry) -> Result<String>
 {
    let e = Entry{
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
    e.validate()?;
    db.create_entry(&e)?;
    Ok(e.id)
}

pub fn update_entry<D: Db>(db: &mut D, e: UpdateEntry) -> Result<()> {
    let old : Entry = db.get_entry(&e.id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let e = Entry{
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
    db.update_entry(&e)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use super::*;

    type RepoResult<T> = result::Result<T, RepoError>;

    pub struct MockDb {
        entries: Vec<Entry>,
        categories: Vec<Category>,
    }

    impl MockDb {
        pub fn new() -> MockDb {
            MockDb {
                entries: vec![],
                categories: vec![]
            }
        }

        pub fn clear_all(&mut self) {
            self.entries.clear();
            self.categories.clear();
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

        fn get_entry(&self, id: &str) -> RepoResult<Entry> {
            get(&self.entries,id)
        }

        fn all_entries(&self) -> RepoResult<Vec<Entry>> {
            Ok(self.entries.clone())
        }

        fn all_categories(&self) -> RepoResult<Vec<Category>> {
            Ok(self.categories.clone())
        }

        fn update_entry(&mut self, e: &Entry) -> RepoResult<()> {
            update(&mut self.entries, e)
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
}
