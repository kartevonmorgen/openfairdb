use crate::core::util::{geo, validate};
use std::collections::HashMap;
use uuid::Uuid;

//TODO: move usecases into separate files

use crate::core::prelude::*;

mod create_new_entry;
mod create_new_user;
mod find_duplicates;
mod login;
mod rate_entry;
mod search;
#[cfg(test)]
pub mod tests;
mod update_entry;

pub use self::create_new_entry::*;
pub use self::create_new_user::*;
pub use self::find_duplicates::*;
pub use self::login::*;
pub use self::rate_entry::*;
pub use self::search::*;
pub use self::update_entry::*;

pub fn get_ratings<D: Db>(db: &D, ids: &[String]) -> Result<Vec<Rating>> {
    Ok(db
        .all_ratings()?
        .iter()
        .filter(|x| ids.iter().any(|id| *id == x.id))
        .cloned()
        .collect())
}

pub fn get_ratings_by_entry_ids<D: Db>(
    db: &D,
    ids: &[String],
) -> Result<HashMap<String, Vec<Rating>>> {
    let ratings = db.all_ratings()?;
    Ok(ids
        .iter()
        .map(|e_id| {
            (
                e_id.clone(),
                ratings
                    .iter()
                    .filter(|r| r.entry_id == **e_id)
                    .cloned()
                    .collect(),
            )
        })
        .collect())
}

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

pub fn get_entries<D: Db>(db: &D, ids: &[String]) -> Result<Vec<Entry>> {
    let entries = db
        .all_entries()?
        .into_iter()
        .filter(|e| ids.iter().any(|id| *id == e.id))
        .collect();
    Ok(entries)
}

pub fn get_user<D: Db>(
    db: &mut D,
    logged_in_username: &str,
    requested_username: &str,
) -> Result<(String, String)> {
    let u: User = db.get_user(requested_username)?;
    if logged_in_username != requested_username {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    Ok((u.username, u.email))
}

pub fn delete_user(db: &mut Db, login_id: &str, u_id: &str) -> Result<()> {
    if login_id != u_id {
        return Err(Error::Parameter(ParameterError::Forbidden));
    }
    db.delete_user(login_id)?;
    Ok(())
}

pub fn subscribe_to_bbox(coordinates: &[Coordinate], username: &str, db: &mut Db) -> Result<()> {
    if coordinates.len() != 2 {
        return Err(Error::Parameter(ParameterError::Bbox));
    }
    let bbox = Bbox {
        south_west: coordinates[0].clone(),
        north_east: coordinates[1].clone(),
    };
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

pub fn bbox_subscriptions_by_coordinate(
    db: &mut Db,
    x: &Coordinate,
) -> Result<Vec<BboxSubscription>> {
    Ok(db
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| geo::is_in_bbox(&x.lat, &x.lng, &s.bbox))
        .collect())
}

pub fn email_addresses_from_subscriptions(
    db: &mut Db,
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

pub fn email_addresses_by_coordinate(db: &mut Db, lat: &f64, lng: &f64) -> Result<Vec<String>> {
    let subs = bbox_subscriptions_by_coordinate(
        db,
        &Coordinate {
            lat: *lat,
            lng: *lng,
        },
    )?;
    let addresses = email_addresses_from_subscriptions(db, &subs)?;
    Ok(addresses)
}
