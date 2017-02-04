#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub id        : String,
    pub created   : u64,
    pub version   : u64,
    pub name      : String
}
