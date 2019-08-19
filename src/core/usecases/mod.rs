use crate::core::{
    prelude::*,
    util::{
        geo::{MapBbox, MapPoint},
        validate,
    },
};
use uuid::Uuid;

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
mod email_token_credentials;
mod find_duplicates;
mod indexing;
mod login;
mod query_events;
mod rate_entry;
mod register;
mod search;
#[cfg(test)]
pub mod tests;
mod update_entry;
mod update_event;

pub use self::{
    archive_comments::*, archive_entries::*, archive_events::*, archive_ratings::*,
    change_user_role::*, confirm_email::*, confirm_email_and_reset_password::*,
    create_new_entry::*, create_new_event::*, create_new_user::*, delete_event::*,
    email_token_credentials::*, find_duplicates::*, indexing::*, login::*, query_events::*,
    rate_entry::*, register::*, search::*, update_entry::*, update_event::*,
};

pub fn load_ratings_with_comments<D: Db>(
    db: &D,
    rating_ids: &[&str],
) -> Result<Vec<(Rating, Vec<Comment>)>> {
    let ratings = db.load_ratings(&rating_ids)?;
    let results = db.zip_ratings_with_comments(ratings)?;
    Ok(results)
}

pub fn get_user<D: Db>(
    db: &D,
    logged_in_username: &str,
    requested_username: &str,
) -> Result<(String, String)> {
    let u: User = db.get_user(requested_username)?;
    if logged_in_username != requested_username {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    Ok((u.username, u.email))
}

pub fn get_event<D: Db>(db: &D, id: &str) -> Result<Event> {
    let mut e: Event = db.get_event(id)?;
    if let Some(ref username) = e.created_by {
        let u = db.get_user(username)?;
        e.created_by = Some(u.email);
    }
    Ok(e)
}

pub fn delete_user(db: &mut dyn Db, login_id: &str, u_id: &str) -> Result<()> {
    if login_id != u_id {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    db.delete_user(login_id)?;
    Ok(())
}

pub fn subscribe_to_bbox(bbox: MapBbox, username: &str, db: &mut dyn Db) -> Result<()> {
    validate::bbox(&bbox)?;

    // TODO: support multiple subscriptions in KVM (frontend)
    // In the meanwile we just replace existing subscriptions
    // with a new one.
    unsubscribe_all_bboxes_by_username(db, username)?;

    let id = Uuid::new_v4().to_simple_ref().to_string();
    db.create_bbox_subscription(&BboxSubscription {
        id,
        bbox,
        username: username.into(),
    })?;
    Ok(())
}

pub fn get_bbox_subscriptions(username: &str, db: &dyn Db) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.username == username)
        .collect())
}

pub fn unsubscribe_all_bboxes_by_username(db: &mut dyn Db, username: &str) -> Result<()> {
    let user_subscriptions: Vec<_> = db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.username == username)
        .map(|s| s.id)
        .collect();
    for s_id in user_subscriptions {
        db.delete_bbox_subscription(&s_id)?;
    }
    Ok(())
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

pub fn email_addresses_from_subscriptions(
    db: &dyn Db,
    subs: &[BboxSubscription],
) -> Result<Vec<String>> {
    let usernames: Vec<_> = subs.iter().map(|s| &s.username).collect();

    let mut addresses: Vec<_> = db
        .all_users()?
        .into_iter()
        .filter(|u| usernames.iter().any(|x| **x == u.username))
        .map(|u| u.email)
        .collect();
    addresses.dedup();
    Ok(addresses)
}

pub fn email_addresses_by_coordinate(db: &dyn Db, pos: MapPoint) -> Result<Vec<String>> {
    let subs = bbox_subscriptions_by_coordinate(db, pos)?;
    let addresses = email_addresses_from_subscriptions(db, &subs)?;
    Ok(addresses)
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
    org: &Option<Organization>,
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
