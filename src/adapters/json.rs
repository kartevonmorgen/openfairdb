use entities as e;

#[derive(Serialize)]
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
    pub ratings     : Vec<String>,
    pub license     : Option<String>,
}

#[derive(Serialize,Deserialize)]
pub struct Rating {
    pub id          : String,
    pub title       : String,
    pub created     : u64,
    pub user        : Option<String>,
    pub value       : i8,
    pub context     : e::RatingContext,
    pub comments    : Vec<Comment>,
    pub source      : String
}

#[derive(Serialize,Deserialize)]
pub struct Comment {
    pub id          : String,
    pub created     : u64,
    pub text        : String,
    pub user        : Option<String>
}

#[derive(Serialize)]
pub struct SearchResult {
    pub visible   : Vec<String>,
    pub invisible : Vec<String>
}

#[derive(Serialize)]
pub struct User {
    pub username: String,
    pub email: String
}

// Entity -> JSON

impl Entry {
    pub fn from_entry_with_tags_and_ratings(e: e::Entry, tags: Vec<e::Tag>, ratings: Vec<e::Rating>) -> Entry {
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
            ratings     : ratings.into_iter().map(|r|r.id).collect(),
            license     : e.license,
        }
    }
}
