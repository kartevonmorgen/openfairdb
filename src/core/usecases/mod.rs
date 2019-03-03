use crate::core::{
    prelude::*,
    util::{
        geo::{MapBbox, MapPoint},
        validate,
    },
};
use std::collections::HashMap;
use uuid::Uuid;

//TODO: move usecases into separate files

mod confirm_email;
mod create_new_entry;
mod create_new_event;
pub mod create_new_user;
mod delete_event;
mod find_duplicates;
mod indexing;
mod login;
mod query_events;
mod rate_entry;
mod search;
#[cfg(test)]
pub mod tests;
mod update_entry;
mod update_event;

pub use self::{
    confirm_email::*, create_new_entry::*, create_new_event::*, create_new_user::*,
    delete_event::*, find_duplicates::*, indexing::*, login::*, query_events::*, rate_entry::*,
    search::*, update_entry::*, update_event::*,
};

pub fn get_comments_by_rating_ids<D: Db>(
    db: &D,
    ids: &[String],
) -> Result<HashMap<String, Vec<Comment>>> {
    let comments = db.all_comments()?;
    Ok(ids
        .iter()
        .map(|r_id| {
            (
                r_id.clone(),
                comments
                    .iter()
                    .filter_map(|comment| {
                        if comment.rating_id == *r_id {
                            Some(comment)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect(),
            )
        })
        .collect())
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

pub fn delete_user(db: &mut Db, login_id: &str, u_id: &str) -> Result<()> {
    if login_id != u_id {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    db.delete_user(login_id)?;
    Ok(())
}

pub fn subscribe_to_bbox(bbox: MapBbox, username: &str, db: &mut Db) -> Result<()> {
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

pub fn get_bbox_subscriptions(username: &str, db: &Db) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.username == username)
        .collect())
}

pub fn unsubscribe_all_bboxes_by_username(db: &mut Db, username: &str) -> Result<()> {
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

pub fn bbox_subscriptions_by_coordinate(db: &Db, pos: &MapPoint) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.bbox.contains_point(&pos))
        .collect())
}

pub fn email_addresses_from_subscriptions(
    db: &Db,
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

pub fn email_addresses_by_coordinate(db: &Db, pos: &MapPoint) -> Result<Vec<String>> {
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
        .map(|t| t.split(" ").map(|x| x.to_owned()).collect::<Vec<_>>())
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

pub fn check_for_owned_tags<D: Db>(
    db: &D,
    tags: &[String],
    org: &Option<Organization>,
) -> Result<()> {
    let owned_tags = db.get_all_tags_owned_by_orgs()?;
    for t in tags {
        if owned_tags.iter().any(|id| id == t) {
            match org {
                Some(ref o) => {
                    if !o.owned_tags.iter().any(|x| x == t) {
                        return Err(ParameterError::OwnedTag.into());
                    }
                }
                None => {
                    return Err(ParameterError::OwnedTag.into());
                }
            }
        }
    }
    Ok(())
}
