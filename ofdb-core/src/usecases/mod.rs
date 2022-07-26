use crate::{
    entities::*,
    util::{
        geo::{MapBbox, MapPoint},
        parse::parse_url_param,
        validate,
    },
};

mod archive_comments;
mod archive_events;
mod archive_ratings;
mod authorize;
mod change_user_role;
pub mod clearance;
mod confirm_email;
mod confirm_email_and_reset_password;
mod create_new_place;
mod create_new_user;
mod delete_event;
mod error;
mod export_event;
mod export_place;
mod filter_event;
mod filter_place;
mod find_duplicates;
mod indexing;
mod load_places;
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

type Result<T> = std::result::Result<T, Error>;

pub use self::{
    archive_comments::*, archive_events::*, archive_ratings::*, authorize::*, change_user_role::*,
    confirm_email::*, confirm_email_and_reset_password::*, create_new_place::*, create_new_user::*,
    delete_event::*, error::Error, export_event::*, export_place::*, filter_event::*,
    filter_place::*, find_duplicates::*, indexing::*, load_places::*, login::*, query_events::*,
    rate_place::*, register::*, review_places::*, search::*, store_event::*, update_place::*,
    user_tokens::*,
};

mod prelude {
    pub use super::error::Error;
    pub type Result<T> = std::result::Result<T, Error>;
    pub use crate::{db::*, entities::*, repositories::*};
}
use self::prelude::*;

//TODO: move usecases into separate files

pub fn load_ratings_with_comments<R>(
    repo: &R,
    rating_ids: &[&str],
) -> Result<Vec<(Rating, Vec<Comment>)>>
where
    R: RatingRepository + CommentRepository,
{
    let ratings = repo.load_ratings(rating_ids)?;
    let results = repo.zip_ratings_with_comments(ratings)?;
    Ok(results)
}

pub fn get_user<R>(repo: &R, logged_in_email: &str, requested_email: &str) -> Result<User>
where
    R: UserRepo,
{
    if logged_in_email != requested_email {
        return Err(Error::Forbidden);
    }
    Ok(repo.get_user_by_email(requested_email)?)
}

pub fn get_event<R: EventRepo>(repo: &R, id: &str) -> Result<Event> {
    Ok(repo.get_event(id)?)
}

#[derive(Clone, Debug, Default)]
pub struct EventQuery {
    pub bbox: Option<MapBbox>,
    pub created_by: Option<Email>,
    pub start_min: Option<Timestamp>,
    pub start_max: Option<Timestamp>,
    pub end_min: Option<Timestamp>,
    pub end_max: Option<Timestamp>,
    pub tags: Option<Vec<String>>,
    pub text: Option<String>,

    pub limit: Option<usize>,
}

impl EventQuery {
    pub fn is_empty(&self) -> bool {
        let Self {
            bbox,
            created_by,
            start_min,
            start_max,
            end_min,
            end_max,
            tags,
            text,
            limit,
        } = self;
        bbox.is_none()
            && created_by.is_none()
            && start_min.is_none()
            && start_max.is_none()
            && end_min.is_none()
            && end_max.is_none()
            && tags.is_none()
            && text.is_none()
            && limit.is_none()
    }
}

pub fn delete_user<R>(repo: &R, login_email: &str, email: &str) -> Result<()>
where
    R: UserRepo,
{
    if login_email != email {
        return Err(Error::Forbidden);
    }
    Ok(repo.delete_user_by_email(email)?)
}

pub fn subscribe_to_bbox<R>(repo: &R, user_email: String, bbox: MapBbox) -> Result<()>
where
    R: SubscriptionRepo + UserRepo,
{
    if !validate::is_valid_bbox(&bbox) {
        return Err(Error::Bbox);
    }

    // TODO: support multiple subscriptions in KVM (frontend)
    // In the meanwhile we just replace existing subscriptions
    // with a new one.
    unsubscribe_all_bboxes(repo, &user_email)?;

    let id = Id::new();
    repo.create_bbox_subscription(&BboxSubscription {
        id,
        user_email,
        bbox,
    })?;
    Ok(())
}

pub fn unsubscribe_all_bboxes<R>(repo: &R, user_email: &str) -> Result<()>
where
    R: SubscriptionRepo,
{
    Ok(repo.delete_bbox_subscriptions_by_email(user_email)?)
}

pub fn get_bbox_subscriptions<R>(repo: &R, user_email: &str) -> Result<Vec<BboxSubscription>>
where
    R: SubscriptionRepo,
{
    Ok(repo
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.user_email == user_email)
        .collect())
}

pub fn bbox_subscriptions_by_coordinate<R>(repo: &R, pos: MapPoint) -> Result<Vec<BboxSubscription>>
where
    R: SubscriptionRepo,
{
    Ok(repo
        .all_bbox_subscriptions()?
        .into_iter()
        .filter(|s| s.bbox.contains_point(pos))
        .collect())
}

pub fn email_addresses_by_coordinate<R>(repo: &R, pos: MapPoint) -> Result<Vec<String>>
where
    R: SubscriptionRepo,
{
    Ok(bbox_subscriptions_by_coordinate(repo, pos)?
        .into_iter()
        .map(|s| s.user_email)
        .collect())
}

pub fn prepare_tag_list<'a>(tags: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut tags: Vec<_> = tags
        .into_iter()
        // Split by whitespace
        .flat_map(|t| t.split_whitespace())
        // Remove reserved character
        .map(|t| t.replace('#', ""))
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

#[derive(Debug, Clone)]
pub struct CustomLinkParam {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl From<CustomLink> for CustomLinkParam {
    fn from(from: CustomLink) -> Self {
        let CustomLink {
            url,
            title,
            description,
        } = from;
        Self {
            url: url.to_string(),
            title,
            description,
        }
    }
}

fn parse_custom_link_param(from: CustomLinkParam) -> Result<CustomLink> {
    let CustomLinkParam {
        url,
        title,
        description,
    } = from;
    let url = parse_url_param(&url)?.ok_or(Error::Url)?;
    Ok(CustomLink {
        url,
        title,
        description,
    })
}
