use super::error::{Error, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Repo;
use super::validate::Validate;
use uuid::Uuid;

type Result<T> = result::Result<T,Error>;

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
        license     :  Some(e.license)    
    };
    e.validate()?;
    r.create(&e)?;
    Ok(e.id)
}

#[cfg(test)]
mod tests {

    use super::*;

    type RepoResult<T> = result::Result<T, RepoError>;

    struct MockRepo<T> {
        objects: Vec<T>,
    }

    impl Repo<Entry> for MockRepo<Entry> {
        type Id = String;

        fn get(&self, id: Self::Id) -> RepoResult<Entry> {
            match self.objects.iter().find(|x| x.id == id) {
                Some(x) => Ok(x.clone()),
                None => Err(RepoError::NotFound),
            }
        }

        fn all(&self) -> RepoResult<Vec<Entry>> {
            Ok(self.objects.clone())
        }

        fn create(&mut self, e: &Entry) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
            }
            Ok(())
        }

        fn update(&mut self, e: &Entry) -> RepoResult<()> {
            if let Some(pos) = self.objects.iter().position(|x| x.id == e.id) {
                self.objects[pos] = e.clone();
            } else {
                self.objects.push(e.clone());
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
            license     : "CC0-1.0".into()
        };
        let mut mock_db: MockRepo<Entry> = MockRepo { objects: vec![] };
        assert!(create_new_entry(&mut mock_db, x).is_err());
    }
}
