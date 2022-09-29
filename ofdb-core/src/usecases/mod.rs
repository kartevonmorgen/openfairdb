use crate::{entities::*, util::parse::parse_url_param};

mod archive_comments;
mod archive_events;
mod archive_ratings;
mod authorize;
mod bbox_subscriptions_by_coordinate;
mod change_user_role;
pub mod clearance;
mod confirm_email;
mod confirm_email_and_reset_password;
mod create_new_place;
mod create_new_user;
mod delete_event;
mod delete_user;
mod email_addresses_by_coordinate;
mod error;
mod export_event;
mod export_place;
mod filter_event;
mod filter_place;
mod find_duplicates;
mod get_bbox_subscriptions;
mod get_event;
mod get_user;
mod indexing;
mod load_places;
mod load_ratings_with_comments;
mod login;
mod query_events;
mod rate_place;
mod register;
mod review_places;
mod search;
mod send_update_reminders;
mod store_event;
mod subscribe_to_bbox;
mod unsubscribe_all_bboxes;
mod update_place;
mod user_tokens;

#[cfg(test)]
pub mod tests;

type Result<T> = std::result::Result<T, Error>;

pub use self::{
    archive_comments::*, archive_events::*, archive_ratings::*, authorize::*,
    bbox_subscriptions_by_coordinate::*, change_user_role::*, confirm_email::*,
    confirm_email_and_reset_password::*, create_new_place::*, create_new_user::*, delete_event::*,
    delete_user::*, email_addresses_by_coordinate::*, error::Error, export_event::*,
    export_place::*, filter_event::*, filter_place::*, find_duplicates::*,
    get_bbox_subscriptions::*, get_event::*, get_user::*, indexing::*, load_places::*,
    load_ratings_with_comments::*, login::*, query_events::*, rate_place::*, register::*,
    review_places::*, search::*, send_update_reminders::*, store_event::*, subscribe_to_bbox::*,
    unsubscribe_all_bboxes::*, update_place::*, user_tokens::*,
};

mod prelude {
    pub use super::error::Error;
    pub type Result<T> = std::result::Result<T, Error>;
    pub use crate::{db::*, entities::*, repositories::*};
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
