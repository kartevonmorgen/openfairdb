use ofdb_entities::{id::Id, organization::ModeratedTag};

use std::result::Result as StdResult;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("addition of moderated tag #{tag:} not permitted")]
    AddNotAllowed { tag: String },

    #[error("removal of moderated tag #{tag:} not permitted")]
    RemoveNotAllowed { tag: String },
}

pub type Result<T> = StdResult<T, Error>;

// Checks if the addition and removal of tags is permitted by organizations.
//
// Returns a list with the ids of organizations that require clearance of
// pending changes.
pub fn authorize_editing_of_tagged_entry<T>(
    moderated_tags_by_org: T,
    old_tags: &[String],
    new_tags: &[String],
) -> Result<Vec<Id>>
where
    T: IntoIterator<Item = (Id, ModeratedTag)>,
{
    let mut clearance_org_ids = Vec::new();
    for (org_id, moderated_tag) in moderated_tags_by_org.into_iter() {
        for added_tag in added_tags(old_tags, new_tags) {
            if &moderated_tag.label != added_tag {
                continue;
            }
            if !moderated_tag.allow_add {
                return Err(Error::AddNotAllowed {
                    tag: added_tag.clone(),
                });
            }
            if moderated_tag.require_clearance {
                clearance_org_ids.push(org_id.clone());
            }
        }
        for removed_tag in removed_tags(old_tags, new_tags) {
            if &moderated_tag.label != removed_tag {
                continue;
            }
            if !moderated_tag.allow_remove {
                return Err(Error::RemoveNotAllowed {
                    tag: removed_tag.clone(),
                });
            }
            if moderated_tag.require_clearance {
                clearance_org_ids.push(org_id.clone());
            }
        }
    }
    clearance_org_ids.sort_unstable();
    clearance_org_ids.dedup();
    Ok(clearance_org_ids)
}

fn added_tags<'a>(
    old_tags: &'a [String],
    new_tags: &'a [String],
) -> impl Iterator<Item = &'a String> {
    new_tags
        .iter()
        .filter(move |new_tag| !old_tags.iter().any(|old_tag| old_tag == *new_tag))
}

fn removed_tags<'a>(
    old_tags: &'a [String],
    new_tags: &'a [String],
) -> impl Iterator<Item = &'a String> {
    old_tags
        .iter()
        .filter(move |old_tag| !new_tags.iter().any(|new_tag| new_tag == *old_tag))
}
