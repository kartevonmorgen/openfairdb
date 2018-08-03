use chrono::*;
use core::prelude::*;
use core::util::validate::Validate;
use uuid::Uuid;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Deserialize, Debug, Clone)]
pub struct NewEntry {
    pub title       : String,
    pub description : String,
    pub lat         : f64,
    pub lng         : f64,
    pub street      : Option<String>,
    pub zip         : Option<String>,
    pub city        : Option<String>,
    pub country     : Option<String>,
    pub email       : Option<String>,
    pub telephone   : Option<String>,
    pub homepage    : Option<String>,
    pub categories  : Vec<String>,
    pub tags        : Vec<String>,
    pub license     : String,
}

pub fn create_new_entry<D: Db>(db: &mut D, e: NewEntry) -> Result<String> {
    let mut tags: Vec<_> = e.tags.into_iter().map(|t| t.replace("#", "")).collect();
    tags.dedup();

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let new_entry = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        osm_node    :  None,
        created     :  Utc::now().timestamp() as u64,
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
        tags,
        license     :  Some(e.license),
    };
    new_entry.validate()?;
    for t in &new_entry.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.create_entry(&new_entry)?;
    Ok(new_entry.id)
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
    use uuid::Uuid;

    #[test]
    fn create_new_valid_entry() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
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
        #[cfg_attr(rustfmt, rustfmt_skip)]
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
    fn add_new_valid_entry_with_tags() {
        #[cfg_attr(rustfmt, rustfmt_skip)]
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
    }

}
