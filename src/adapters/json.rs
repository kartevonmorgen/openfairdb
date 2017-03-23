use entities as e;
use std::convert::TryFrom;
use adapters::error::ConversionError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub id          : Option<String>,
    pub created     : Option<u64>,
    pub version     : Option<u64>,
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
    pub categories  : Option<Vec<String>>,
    pub tags        : Vec<String>,
    pub license     : Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id        : Option<String>,
    pub created   : Option<u64>,
    pub version   : Option<u64>,
    pub name      : Option<String>
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
            id          : Some(e.id),
            created     : Some(e.created),
            version     : Some(e.version),
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
            categories  : Some(e.categories),
            tags        : tags.into_iter().map(|e|e.id).collect(),
            license     : e.license
        }
    }
}

impl From<e::Category> for Category {
    fn from(c: e::Category) -> Category {
        Category{
            id          : Some(c.id),
            created     : Some(c.created),
            version     : Some(c.version),
            name        : Some(c.name),
        }
    }
}

// JSON -> Entity

impl TryFrom<Entry> for e::Entry {
    type Error = ConversionError;
    fn try_from(e: Entry) -> Result<e::Entry, ConversionError> {
        Ok(e::Entry{
            id          : e.id.ok_or_else(||ConversionError::Id)?,
            created     : e.created.ok_or_else(||ConversionError::Created)?,
            version     : e.version.ok_or_else(||ConversionError::Version)?,
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
            categories  : e.categories.ok_or_else(||ConversionError::Categories)?,
            license     : e.license
        })
    }
}

impl TryFrom<Category> for e::Category {
    type Error = ConversionError;
    fn try_from(c: Category) -> Result<e::Category,ConversionError> {
        Ok(e::Category{
            id          : c.id.ok_or_else(||ConversionError::Id)?,
            created     : c.created.ok_or_else(||ConversionError::Created)?,
            version     : c.version.ok_or_else(||ConversionError::Version)?,
            name        : c.name.ok_or_else(||ConversionError::Name)?,
        })
    }
}
