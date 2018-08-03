use chrono::*;
use core::prelude::*;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    pub id          : String,
    pub osm_node    : Option<u64>,
    pub version     : u64,
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
}

pub fn update_entry<D: Db>(db: &mut D, e: UpdateEntry) -> Result<()> {
    let old: Entry = db.get_entry(&e.id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion));
    }
    let mut tags = e.tags;
    tags.dedup();
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let new_entry = Entry{
        id          :  e.id,
        osm_node    :  None,
        created     :  Utc::now().timestamp() as u64,
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
        tags,
        license     :  old.license, // license is immutable
    };
    for t in &new_entry.tags {
        db.create_tag_if_it_does_not_exist(&Tag { id: t.clone() })?;
    }
    db.update_entry(&new_entry)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::*;
    use uuid::Uuid;

    #[test]
    fn update_valid_entry() {
        let id = Uuid::new_v4().simple().to_string();
        let old = Entry::build()
            .id(&id)
            .version(1)
            .title("foo")
            .description("bar")
            .finish();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            id          : id.clone(),
            osm_node    :  None,
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
        let old = Entry::build()
            .id(&id)
            .version(3)
            .title("foo")
            .description("bar")
            .finish();

        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            id          : id.clone(),
            osm_node    :  None,
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
            Error::Repo(err) => match err {
                RepoError::InvalidVersion => {}
                _ => {
                    panic!("invalid error type");
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.entries.len(), 1);
    }

    #[test]
    fn update_non_existing_entry() {
        let id = Uuid::new_v4().simple().to_string();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            id          : id.clone(),
            osm_node    :  None,
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
            Error::Repo(err) => match err {
                RepoError::NotFound => {}
                _ => {
                    panic!("invalid error type");
                }
            },
            _ => {
                panic!("invalid error type");
            }
        }
        assert_eq!(mock_db.entries.len(), 0);
    }

    #[test]
    fn update_valid_entry_with_tags() {
        let id = Uuid::new_v4().simple().to_string();
        let old = Entry::build()
            .id(&id)
            .version(1)
            .tags(vec!["bio", "fair"])
            .finish();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let new = UpdateEntry {
            id          : id.clone(),
            osm_node    :  None,
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
        mock_db.tags = vec![Tag { id: "bio".into() }, Tag { id: "fair".into() }];
        assert!(update_entry(&mut mock_db, new).is_ok());
        let e = mock_db.get_entry(&id).unwrap();
        assert_eq!(e.tags, vec!["vegan"]);
        assert_eq!(mock_db.tags.len(), 3);
    }

}
