#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tag {
    pub id: String,
}

pub type TagCount = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagFrequency(pub String, pub TagCount);
