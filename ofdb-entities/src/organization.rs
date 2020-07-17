use crate::id::Id;

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: Id,
    pub name: String,
    pub owned_tags: Vec<String>,
    pub api_token: String,
}
