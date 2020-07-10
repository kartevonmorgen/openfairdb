use crate::core::{
    prelude::*,
    util::{
        geo::{MapBbox, MapPoint},
        validate,
    },
};

mod archive_comments;
mod archive_events;
mod archive_ratings;
mod authorize_organization;
mod change_user_role;
mod confirm_email;
mod confirm_email_and_reset_password;
mod create_new_place;
mod create_new_user;
mod delete_event;
mod export_event;
mod export_place;
mod filter_event;
mod filter_place;
mod find_duplicates;
mod indexing;
mod login;
mod query_events;
mod rate_place;
mod register;
mod review_places;
mod search;
mod store_event;
mod update_place;
mod user_tokens;

#[cfg(test)]
pub mod tests;

pub use self::{
    archive_comments::*, archive_events::*, archive_ratings::*, authorize_organization::*,
    change_user_role::*, confirm_email::*, confirm_email_and_reset_password::*,
    create_new_place::*, create_new_user::*, delete_event::*, export_event::*, export_place::*,
    filter_event::*, filter_place::*, find_duplicates::*, indexing::*, login::*, query_events::*,
    rate_place::*, register::*, review_places::*, search::*, store_event::*, update_place::*,
    user_tokens::*,
};

//TODO: move usecases into separate files

pub fn load_ratings_with_comments<D: Db>(
    db: &D,
    rating_ids: &[&str],
) -> Result<Vec<(Rating, Vec<Comment>)>> {
    let ratings = db.load_ratings(&rating_ids)?;
    let results = db.zip_ratings_with_comments(ratings)?;
    Ok(results)
}

pub fn get_user<D: Db>(db: &D, logged_in_email: &str, requested_email: &str) -> Result<User> {
    if logged_in_email != requested_email {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    Ok(db.get_user_by_email(requested_email)?)
}

pub fn get_event<D: Db>(db: &D, id: &str) -> Result<Event> {
    Ok(db.get_event(id)?)
}

#[derive(Clone, Debug, Default)]
pub struct EventQuery {
    pub bbox: Option<MapBbox>,
    pub created_by: Option<Email>,
    pub start_min: Option<Timestamp>,
    pub start_max: Option<Timestamp>,
    pub tags: Option<Vec<String>>,
    pub text: Option<String>,

    pub limit: Option<usize>,
}

impl EventQuery {
    pub fn is_empty(&self) -> bool {
        let Self {
            ref bbox,
            ref created_by,
            ref start_min,
            ref start_max,
            ref tags,
            ref text,
            ref limit,
        } = self;
        bbox.is_none()
            && created_by.is_none()
            && start_min.is_none()
            && start_max.is_none()
            && tags.is_none()
            && text.is_none()
            && limit.is_none()
    }
}

pub fn delete_user(db: &dyn Db, login_email: &str, email: &str) -> Result<()> {
    if login_email != email {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    Ok(db.delete_user_by_email(email)?)
}

pub fn subscribe_to_bbox(db: &dyn Db, user_email: String, bbox: MapBbox) -> Result<()> {
    validate::bbox(&bbox)?;

    // TODO: support multiple subscriptions in KVM (frontend)
    // In the meanwhile we just replace existing subscriptions
    // with a new one.
    unsubscribe_all_bboxes(db, &user_email)?;

    let id = Id::new();
    db.create_bbox_subscription(&BboxSubscription {
        id,
        user_email,
        bbox,
    })?;
    Ok(())
}

pub fn unsubscribe_all_bboxes(db: &dyn Db, user_email: &str) -> Result<()> {
    Ok(db.delete_bbox_subscriptions_by_email(&user_email)?)
}

pub fn get_bbox_subscriptions(db: &dyn Db, user_email: &str) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.user_email == user_email)
        .collect())
}

pub fn bbox_subscriptions_by_coordinate(
    db: &dyn Db,
    pos: MapPoint,
) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.bbox.contains_point(pos))
        .collect())
}

pub fn email_addresses_by_coordinate(db: &dyn Db, pos: MapPoint) -> Result<Vec<String>> {
    Ok(bbox_subscriptions_by_coordinate(db, pos)?
        .into_iter()
        .map(|s| s.user_email)
        .collect())
}

pub fn prepare_tag_list<'a>(tags: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut tags: Vec<_> = tags
        .into_iter()
        // Split by whitespace
        .flat_map(|t| t.split_whitespace())
        // Convert to lowercase
        .map(|t| t.to_lowercase())
        // Remove reserved character
        .map(|t| t.replace("#", ""))
        // Filter empty tags (2nd pass) and conversion to lowercase
        .filter_map(|t| match t.trim() {
            t if t.is_empty() => None,
            t => Some(t.to_lowercase()),
        })
        .collect();
    tags.sort_unstable();
    tags.dedup();
    tags
}

// Counts and returns the number of tags owned by this org and a list
// of other organizations that require authorization of pending changes.
pub fn authorize_moderated_tags_owned_by_orgs<D: Db>(
    db: &D,
    old_tags: &[String],
    new_tags: &[String],
    org: Option<&Organization>,
) -> Result<Vec<Id>> {
    let mod_tags = db.get_all_tags_owned_by_orgs()?;
    let mut count = 0;
    let mod_tags_of_other_orgs = if let Some(org) = org {
        mod_tags
            .into_iter()
            .filter(|mod_tag| {
                !org.moderated_tags.iter().any(|org_tag| {
                    if org_tag == mod_tag {
                        count += 1;
                        true
                    } else {
                        false
                    }
                })
            })
            .collect()
    } else {
        mod_tags
    };
    let mut auth_org_ids = Vec::new();
    for added_tag in new_tags
        .iter()
        .filter(|new_tag| !old_tags.iter().any(|old_tag| old_tag == *new_tag))
    {
        for mod_tag in mod_tags_of_other_orgs
            .iter()
            .filter(|mod_tag| &mod_tag.label == added_tag)
        {
            if !mod_tag.flags.allows_add() {
                return Err(ParameterError::ModeratedTag.into());
            }
            if mod_tag.flags.requires_authorization() {
                //FIXME: auth_org_ids.push(org_id);
            }
        }
    }
    for removed_tag in old_tags
        .iter()
        .filter(|old_tag| !new_tags.iter().any(|new_tag| new_tag == *old_tag))
    {
        for mod_tag in mod_tags_of_other_orgs
            .iter()
            .filter(|mod_tag| &mod_tag.label == removed_tag)
        {
            if !mod_tag.flags.allows_remove() {
                return Err(ParameterError::ModeratedTag.into());
            }
            if mod_tag.flags.requires_authorization() {
                //FIXME: auth_org_ids.push(org_id);
            }
        }
    }
    auth_org_ids.sort_unstable();
    auth_org_ids.dedup();
    Ok(auth_org_ids)
}

pub fn authorize_user_by_email(
    db: &dyn Db,
    user_email: &str,
    min_required_role: Role,
) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(user_email)? {
        if user.role >= min_required_role {
            return Ok(user);
        }
    }
    Err(Error::Parameter(ParameterError::Unauthorized))
}
