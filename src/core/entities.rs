#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Entry {
    pub id          : String,
    pub osm_node    : Option<u64>,
    pub created     : u64,
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
    pub license     : Option<String>,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Category {
    pub id      : String,
    pub created : u64,
    pub version : u64,
    pub name    : String
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Tag {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ObjectId {
    #[serde(rename = "entry")]
    Entry(String),
    #[serde(rename = "tag")]
    Tag(String),
    #[serde(rename = "user")]
    User(String),
    #[serde(rename = "comment")]
    Comment(String),
    #[serde(rename = "rating")]
    Rating(String),
    #[serde(rename = "bbox_subscription")]
    BboxSubscription(String),
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id              : String, // TODO: remove
    pub username        : String,
    pub password        : String,
    pub email           : String,
    pub email_confirmed : bool,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Comment {
    pub id        : String,
    pub created   : u64,
    pub text      : String,
    pub rating_id : String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum RatingContext {
    #[serde(rename = "diversity")]
    Diversity,
    #[serde(rename = "renewable")]
    Renewable,
    #[serde(rename = "fairness")]
    Fairness,
    #[serde(rename = "humanity")]
    Humanity,
    #[serde(rename = "transparency")]
    Transparency,
    #[serde(rename = "solidarity")]
    Solidarity,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Rating {
    pub id       : String,
    pub entry_id : String,
    pub created  : u64,
    pub title    : String,
    pub value    : i8,
    pub context  : RatingContext,
    pub source   : Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Bbox {
    pub south_west: Coordinate,
    pub north_east: Coordinate,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BboxSubscription {
    pub id       : String,
    pub bbox     : Bbox,
    pub username : String,
}

#[cfg(test)]
pub use self::entry_builder::*;

#[cfg(test)]
pub mod entry_builder {

    use super::*;
    use uuid::Uuid;

    pub trait EntryBuilder {
        fn build() -> EntryBuild;
    }

    pub struct EntryBuild {
        entry: Entry,
    }

    impl EntryBuild {
        pub fn id(mut self, id: &str) -> Self {
            self.entry.id = id.into();
            self
        }
        pub fn version(mut self, v: u64) -> Self {
            self.entry.version = v;
            self
        }
        pub fn title(mut self, title: &str) -> Self {
            self.entry.title = title.into();
            self
        }
        pub fn description(mut self, desc: &str) -> Self {
            self.entry.description = desc.into();
            self
        }
        pub fn lat(mut self, lat: f64) -> Self {
            self.entry.lat = lat;
            self
        }
        pub fn lng(mut self, lng: f64) -> Self {
            self.entry.lng = lng;
            self
        }
        pub fn categories(mut self, cats: Vec<&str>) -> Self {
            self.entry.categories = cats.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn tags(mut self, tags: Vec<&str>) -> Self {
            self.entry.tags = tags.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn license(mut self, license: Option<&str>) -> Self {
            self.entry.license = license.map(|s| s.into());
            self
        }
        pub fn finish(self) -> Entry {
            self.entry
        }
    }

    impl EntryBuilder for Entry {
        fn build() -> EntryBuild {
            EntryBuild {
                entry: Entry::default(),
            }
        }
    }

    impl Default for Entry {
        fn default() -> Entry {
            #[cfg_attr(rustfmt, rustfmt_skip)]
            Entry{
                id          : Uuid::new_v4().simple().to_string(),
                osm_node    : None,
                created     : 0,
                version     : 0,
                title       : "".into(),
                description : "".into(),
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
                license     : None,
            }
        }
    }

}
