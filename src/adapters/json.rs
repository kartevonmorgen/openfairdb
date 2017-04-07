use entities as e;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub id          : String,
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

#[derive(Serialize, Debug, Clone)]
pub struct SearchResult {
    pub visible   : Vec<String>,
    pub invisible : Vec<String>
}

// Entity -> JSON

impl Entry {
    pub fn from_entry_with_tags(e: e::Entry, tags: Vec<e::Tag>) -> Entry {
        Entry{
            id          : e.id,
            created     : e.created,
            version     : e.version,
            title       : e.title,
            description : e.description,
            lat         : e.lat,
            lng         : e.lng,
            street      : e.street,
            zip         : e.zip,
            city        : e.city,
            country     : e.country,
            email       : e.email,
            telephone   : e.telephone,
            homepage    : e.homepage,
            categories  : e.categories,
            tags        : tags.into_iter().map(|e|e.id).collect(),
            license     : e.license,
        }
    }
}
