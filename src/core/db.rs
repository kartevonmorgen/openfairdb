use super::{
    entities::*,
    error::RepoError,
    util::geo::{MapBbox, MapPoint},
};

use failure::Fallible;
use std::result;

type Result<T> = result::Result<T, RepoError>;

pub trait EntryGateway {
    fn get_entry(&self, _: &str) -> Result<Entry>;
    fn get_entries(&self, ids: &[&str]) -> Result<Vec<Entry>>;

    fn all_entries(&self) -> Result<Vec<Entry>>;
    fn count_entries(&self) -> Result<usize>;

    fn create_entry(&self, _: Entry) -> Result<()>;
    fn update_entry(&self, _: &Entry) -> Result<()>;
    fn import_multiple_entries(&mut self, _: &[Entry]) -> Result<()>;
    fn archive_entries(&self, ids: &[&str], archived: u64) -> Result<usize>;
}

pub trait EventGateway {
    fn create_event(&self, _: Event) -> Result<()>;
    fn get_event(&self, _: &str) -> Result<Event>;
    fn all_events(&self) -> Result<Vec<Event>>;
    fn update_event(&self, _: &Event) -> Result<()>;
    fn archive_events(&self, ids: &[&str], archived: u64) -> Result<usize>;
    fn delete_event(&self, _: &str) -> Result<()>;
    //TODO: fn count_events(&self) -> Result<usize>;
}

pub trait UserGateway {
    fn create_user(&mut self, user: User) -> Result<()>;
    fn update_user(&mut self, user: &User) -> Result<()>;
    fn get_user(&self, username: &str) -> Result<User>;
    fn get_user_by_email(&self, email: &str) -> Result<User>;
    fn all_users(&self) -> Result<Vec<User>>;
    fn delete_user(&mut self, username: &str) -> Result<()>;
    //TODO: fn count_users(&self) -> Result<usize>;
}

pub trait CommentRepository {
    fn create_comment(&self, _: Comment) -> Result<()>;

    // Only unarchived comments
    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>>;

    // Only unarchived comments (even if the rating has already been archived)
    fn zip_ratings_with_comments(
        &self,
        ratings: Vec<Rating>,
    ) -> Result<Vec<(Rating, Vec<Comment>)>> {
        let mut results = Vec::with_capacity(ratings.len());
        for rating in ratings {
            debug_assert!(rating.archived.is_none());
            let comments = self.load_comments_of_rating(&rating.id)?;
            results.push((rating, comments));
        }
        Ok(results)
    }

    fn archive_comments(&self, ids: &[&str], archived: u64) -> Result<usize>;
    fn archive_comments_of_ratings(&self, rating_ids: &[&str], archived: u64) -> Result<usize>;
    fn archive_comments_of_entries(&self, entry_ids: &[&str], archived: u64) -> Result<usize>;
}

pub trait OrganizationGateway {
    fn create_org(&mut self, _: Organization) -> Result<()>;
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization>;
    fn get_all_tags_owned_by_orgs(&self) -> Result<Vec<String>>;
}

pub trait RatingRepository {
    fn create_rating(&self, rating: Rating) -> Result<()>;

    // Only unarchived ratings without comments
    fn load_rating(&self, id: &str) -> Result<Rating>;
    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>>;
    fn load_ratings_of_entry(&self, entry_id: &str) -> Result<Vec<Rating>>;

    fn archive_ratings(&self, ids: &[&str], archived: u64) -> Result<usize>;
    fn archive_ratings_of_entries(&self, entry_ids: &[&str], archived: u64) -> Result<usize>;

    fn load_entry_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>>;
}

//TODO:
//  - TagGeatway
//  - CategoryGateway
//  - SubscriptionGateway

pub trait Db:
    EntryGateway
    + UserGateway
    + CommentRepository
    + EventGateway
    + OrganizationGateway
    + RatingRepository
{
    fn create_tag_if_it_does_not_exist(&self, _: &Tag) -> Result<()>;
    fn create_category_if_it_does_not_exist(&mut self, _: &Category) -> Result<()>;
    fn create_bbox_subscription(&mut self, _: &BboxSubscription) -> Result<()>;

    fn all_categories(&self) -> Result<Vec<Category>>;
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn count_tags(&self) -> Result<usize>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;

    fn delete_bbox_subscription(&mut self, _: &str) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct IndexedEntry {
    pub id: String,
    pub pos: MapPoint,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: AvgRatings,
}

#[derive(Debug, Default, Clone)]
pub struct EntryIndexQuery<'a, 'b> {
    pub bbox: Option<MapBbox>,
    pub categories: Vec<&'a str>,
    pub ids: Vec<&'b str>,
    pub tags: Vec<String>,
    pub text: Option<String>,
}

pub trait EntryIndex {
    fn query_entries(&self, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<IndexedEntry>>;
}

pub trait EntryIndexer: EntryIndex {
    fn add_or_update_entry(&mut self, entry: &Entry, ratings: &AvgRatings) -> Fallible<()>;
    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()>;
    fn flush(&mut self) -> Fallible<()>;
}
