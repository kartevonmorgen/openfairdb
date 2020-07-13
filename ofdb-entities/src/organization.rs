use crate::id::Id;

use std::cmp::{Ordering, PartialOrd};

pub type TagModerationFlagsValue = u8;

/// A bit mask with flags that control flags for owned tags.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TagModerationFlags(TagModerationFlagsValue);

impl TagModerationFlags {
    pub const fn none() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self(7)
    }

    /// Allow users to add this tag
    pub const fn add() -> Self {
        Self(1)
    }

    /// Allow users to remove this tag
    pub const fn remove() -> Self {
        Self(2)
    }

    /// Hide edits on entries until authorized
    pub const fn authorize() -> Self {
        Self(4)
    }

    pub fn allows_add(self) -> bool {
        match Self::add().partial_cmp(&self) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    pub fn allows_remove(self) -> bool {
        match Self::remove().partial_cmp(&self) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    pub fn requires_authorization(self) -> bool {
        match Self::authorize().partial_cmp(&self) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    pub fn join(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl PartialOrd for TagModerationFlags {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        if self.intersect(*other) == *self {
            return Some(Ordering::Less);
        }
        if other.intersect(*self) == *other {
            return Some(Ordering::Greater);
        }
        None
    }
}

impl Default for TagModerationFlags {
    fn default() -> Self {
        Self::none()
    }
}

impl From<TagModerationFlags> for TagModerationFlagsValue {
    fn from(from: TagModerationFlags) -> Self {
        from.0
    }
}

impl From<TagModerationFlagsValue> for TagModerationFlags {
    fn from(from: TagModerationFlagsValue) -> Self {
        Self(from)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModeratedTag {
    pub label: String,
    pub moderation_flags: TagModerationFlags,
}

// Workaround for backwards compatbility
// TODO: Remove after updating tests
impl From<&str> for ModeratedTag {
    fn from(from: &str) -> Self {
        Self {
            label: from.to_string(),
            moderation_flags: Default::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_add() {
        assert!(!TagModerationFlags::none().allows_add());
        assert!(TagModerationFlags::add().allows_add());
        assert!(!TagModerationFlags::remove().allows_add());
        assert!(!TagModerationFlags::authorize().allows_add());
        assert!(TagModerationFlags::all().allows_add());
    }
}
