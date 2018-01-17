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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Category {
    pub id        : String,
    pub created   : u64,
    pub version   : u64,
    pub name      : String
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: String, // TODO: remove
    pub username: String,
    pub password: String,
    pub email: String,
    pub email_confirmed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Comment {
    pub id: String,
    pub created: u64,
    pub text: String,
    pub rating_id: String,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Rating {
    pub id: String,
    pub entry_id: String,
    pub created: u64,
    pub title: String,
    pub value: i8,
    pub context: RatingContext,
    pub source: Option<String>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BboxSubscription {
    pub id: String,
    pub bbox: Bbox,
    pub username: String,
}
