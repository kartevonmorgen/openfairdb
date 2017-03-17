#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
    pub id : String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Relation {
    #[serde(rename="is_tagged_with")]
    IsTaggedWith
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Triple {
    pub subject : ObjectId,
    pub predicate : Relation,
    pub object : ObjectId,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum ObjectId {
    #[serde(rename="entry")]
    Entry(String),
    #[serde(rename="tag")]
    Tag(String),
}
