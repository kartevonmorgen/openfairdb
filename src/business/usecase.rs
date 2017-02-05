use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Repo;
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

pub fn create_new_entry<R: Repo<Entry>>(r: &mut R, e: NewEntry) -> Result<String>
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
        tags        :  e.tags,
        license     :  Some(e.license)
    };
    e.validate()?;
    r.create(&e)?;
    Ok(e.id)
}

pub fn update_entry<R: Repo<Entry>>(r: &mut R, e: UpdateEntry) -> Result<()> {
    let old : Entry = r.get(&e.id)?;
    if old.version != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let e = Entry{
        id          :  e.id,
        created     :  UTC::now().timestamp() as u64,
        version     :  e.version+1,
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
        tags        :  e.tags,
        license     :  old.license
    };
    r.update(&e)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {

    use super::*;

    type RepoResult<T> = result::Result<T, RepoError>;

    pub struct MockRepo<T> {
        objects: Vec<T>,
    }

    impl<T> MockRepo<T> {
        pub fn new() -> MockRepo<T> {
            MockRepo {
                objects: vec![]
            }
        }

        pub fn clear(&mut self) {
            self.objects = vec![];
        }
    }

    impl<T:Id + Clone> Repo<T> for MockRepo<T> {

        fn get(&self, id: &str) -> RepoResult<T> {
            match self.objects.iter().find(|x| x.id() == id) {
                Some(x) => Ok(x.clone()),
                None => Err(RepoError::NotFound),
            }
        }

        fn all(&self) -> RepoResult<Vec<T>> {
            Ok(self.objects.clone())
        }

        fn create(&mut self, e: &T) -> RepoResult<()> {
            if self.objects.iter().any(|x| x.id() == e.id()) {
                return Err(RepoError::AlreadyExists)
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }

        fn update(&mut self, e: &T) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id() == e.id()) {
                self.objects[pos] = e.clone();
            } else {
                return Err(RepoError::NotFound)
            }
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
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        let now = UTC::now();
        let id = create_new_entry(&mut mock_db, x).unwrap();
        assert!(Uuid::parse_str(&id).is_ok());
        assert_eq!(mock_db.objects.len(),1);
        let x = &mock_db.objects[0];
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
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
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
            tags        : vec![],
            license     : None
        };
        let new = UpdateEntry {
            id          : id.clone(),
            version     : 1,
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
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![old]};
        let now = UTC::now();
        assert!(update_entry(&mut mock_db, new).is_ok());
        assert_eq!(mock_db.objects.len(),1);
        let x = &mock_db.objects[0];
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
            tags        : vec![],
            license     : None
        };
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
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![old]};
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
        assert_eq!(mock_db.objects.len(),1);
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
        let mut mock_db : MockRepo<Entry> = MockRepo{ objects: vec![]};
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
        assert_eq!(mock_db.objects.len(),0);
    }
}
