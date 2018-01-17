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
}

#[derive(Serialize)]
pub struct EntryIdWithCoordinates {
    pub id : String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub visible   : Vec<EntryIdWithCoordinates>,
    pub invisible : Vec<EntryIdWithCoordinates>
}

#[derive(Serialize)]
pub struct User {
    pub u_id: String,
    pub email: String
}

#[derive(Serialize)]
pub struct BboxSubscription{
    pub id              : String,
    pub south_west_lat  : f64,
    pub south_west_lng  : f64,
    pub north_east_lat  : f64,
    pub north_east_lng  : f64,
}

// Entity -> JSON

impl Entry {
    pub fn from_entry_with_ratings(e: e::Entry, ratings: Vec<e::Rating>) -> Entry {
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
            tags        : e.tags,
            ratings     : ratings.into_iter().map(|r|r.id).collect(),
            license     : e.license,
        }
    }
}
