use crate::core::{
    prelude::*,
    util::{
        geo::{MapBbox, MapPoint},
        validate,
    },
};

//TODO: move usecases into separate files

mod archive_comments;
mod archive_entries;
mod archive_events;
mod archive_ratings;
mod change_user_role;
mod confirm_email;
mod confirm_email_and_reset_password;
mod create_new_entry;
mod create_new_event;
pub mod create_new_user;
mod delete_event;
mod find_duplicates;
mod indexing;
mod login;
mod query_events;
mod rate_entry;
mod register;
mod search;
mod update_entry;
mod update_event;
mod user_tokens;

#[cfg(test)]
pub mod tests;

pub use self::{
    archive_comments::*, archive_entries::*, archive_events::*, archive_ratings::*,
    change_user_role::*, confirm_email::*, confirm_email_and_reset_password::*,
    create_new_entry::*, create_new_event::*, create_new_user::*, delete_event::*,
    find_duplicates::*, indexing::*, login::*, query_events::*, rate_entry::*, register::*,
    search::*, update_entry::*, update_event::*, user_tokens::*,
};

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

pub fn get_event<D: Db>(db: &D, uid: &str) -> Result<Event> {
    Ok(db.get_event(uid)?)
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

    let uid = Uid::new_uuid();
    db.create_bbox_subscription(&BboxSubscription {
        uid,
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

pub fn prepare_tag_list(tags: Vec<String>) -> Vec<String> {
    let mut tags: Vec<_> = tags
        .into_iter()
        // Filter empty tags (1st pass)
        .filter_map(|t| match t.trim() {
            t if t.is_empty() => None,
            t => Some(t.to_owned()),
        })
        // Split and recollect
        .map(|t| t.split(' ').map(ToOwned::to_owned).collect::<Vec<_>>())
        .flatten()
        // Remove reserved character
        .map(|t| t.replace("#", ""))
        // Filter empty tags (2nd pass)
        .filter_map(|t| match t.trim() {
            t if t.is_empty() => None,
            t => Some(t.to_owned()),
        })
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

// Counts and returns the number of tags owned by this org. If the
// given list of tags contains tags that are owned by any other org
// then fails with ParameterError::OwnedTag.
pub fn check_and_count_owned_tags<D: Db>(
    db: &D,
    tags: &[String],
    org: Option<&Organization>,
) -> Result<usize> {
    let owned_tags = db.get_all_tags_owned_by_orgs()?;
    let mut count = 0;
    for t in tags {
        if owned_tags.iter().any(|id| id == t) {
            match org {
                Some(ref o) => {
                    if !o.owned_tags.iter().any(|x| x == t) {
                        return Err(ParameterError::OwnedTag.into());
                    }
                    count += 1;
                }
                None => {
                    return Err(ParameterError::OwnedTag.into());
                }
            }
        }
    }
    Ok(count)
}

pub fn authorize_user_by_email(db: &dyn Db, user_email: &str, min_required_role: Role) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(user_email)? {
        if user.role >= min_required_role {
            return Ok(user);
        }
    }
    Err(Error::Parameter(ParameterError::Unauthorized))
}
