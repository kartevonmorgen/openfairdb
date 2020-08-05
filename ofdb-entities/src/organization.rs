use crate::id::Id;

#[derive(Debug, Clone, PartialEq)]
pub struct ModeratedTag {
    pub label: String,
    pub allow_add: bool,
    pub allow_remove: bool,
    pub require_clearance: bool,
}

// Workaround for backwards compatbility
// TODO: Remove after updating tests
impl From<&str> for ModeratedTag {
    fn from(from: &str) -> Self {
        Self {
            label: from.to_string(),
            allow_add: false,
            allow_remove: false,
            require_clearance: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: Id,
    pub name: String,
    pub api_token: String,
    pub moderated_tags: Vec<ModeratedTag>,
}
