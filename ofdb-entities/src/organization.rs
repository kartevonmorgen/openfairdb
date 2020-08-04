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

    /// Allow users to add this tag to entries
    pub const fn allow_adding_of_tag() -> Self {
        Self(1)
    }

    /// Allow users to remove this tag from entries
    pub const fn allow_removal_of_tag() -> Self {
        Self(2)
    }

    /// Hide edits of entries until cleared by the moderating organization
    pub const fn require_clearance_by_organization() -> Self {
        Self(4)
    }

    pub fn allows_adding_of_tag(self) -> bool {
        match Self::allow_adding_of_tag().partial_cmp(&self) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    pub fn allows_removal_of_tag(self) -> bool {
        match Self::allow_removal_of_tag().partial_cmp(&self) {
            Some(Ordering::Less) | Some(Ordering::Equal) => true,
            _ => false,
        }
    }

    pub fn requires_clearance_by_organization(self) -> bool {
        match Self::require_clearance_by_organization().partial_cmp(&self) {
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
    fn allows_adding_of_tag() {
        assert!(!TagModerationFlags::none().allows_adding_of_tag());
        assert!(TagModerationFlags::allow_adding_of_tag().allows_adding_of_tag());
        assert!(!TagModerationFlags::allow_removal_of_tag().allows_adding_of_tag());
        assert!(!TagModerationFlags::require_clearance_by_organization().allows_adding_of_tag());
        assert!(TagModerationFlags::all().allows_adding_of_tag());
    }

    #[test]
    fn allows_removal_of_tag() {
        assert!(!TagModerationFlags::none().allows_removal_of_tag());
        assert!(!TagModerationFlags::allow_adding_of_tag().allows_removal_of_tag());
        assert!(TagModerationFlags::allow_removal_of_tag().allows_removal_of_tag());
        assert!(!TagModerationFlags::require_clearance_by_organization().allows_removal_of_tag());
        assert!(TagModerationFlags::all().allows_removal_of_tag());
    }

    #[test]
    fn requires_clearance_by_organization() {
        assert!(!TagModerationFlags::none().requires_clearance_by_organization());
        assert!(!TagModerationFlags::allow_adding_of_tag().requires_clearance_by_organization());
        assert!(!TagModerationFlags::allow_removal_of_tag().requires_clearance_by_organization());
        assert!(TagModerationFlags::require_clearance_by_organization()
            .requires_clearance_by_organization());
        assert!(TagModerationFlags::all().requires_clearance_by_organization());
    }
}
