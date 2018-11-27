use super::{entities::*, error::RepoError};
use std::result;

type Result<T> = result::Result<T, RepoError>;

pub trait EntryGateway {
    fn create_entry(&mut self, _: &Entry) -> Result<()>;
    fn get_entry(&self, _: &str) -> Result<Entry>;
    fn get_entries_by_bbox(&self, _: &Bbox) -> Result<Vec<Entry>>;
    fn all_entries(&self) -> Result<Vec<Entry>>;
    fn update_entry(&mut self, _: &Entry) -> Result<()>;
    fn import_multiple_entries(&mut self, _: &[Entry]) -> Result<()>;
}

pub trait UserGateway {
    fn create_user(&mut self, _: &User) -> Result<()>;
    fn update_user(&mut self, _: &User) -> Result<()>;
    fn get_user(&self, _: &str) -> Result<User>;
    fn all_users(&self) -> Result<Vec<User>>;
    fn delete_user(&mut self, _: &str) -> Result<()>;
}

pub trait CommentGateway {
    fn create_comment(&mut self, _: &Comment) -> Result<()>;
    fn all_comments(&self) -> Result<Vec<Comment>>;
}

//TODO:
//  - TagGeatway
//  - CategoryGateway
//  - RatingGateway
//  - SubscriptionGateway

pub trait Db: EntryGateway + UserGateway + CommentGateway {
    fn create_tag_if_it_does_not_exist(&mut self, _: &Tag) -> Result<()>;
    fn create_category_if_it_does_not_exist(&mut self, _: &Category) -> Result<()>;
    fn create_rating(&mut self, _: &Rating) -> Result<()>;
    fn create_bbox_subscription(&mut self, _: &BboxSubscription) -> Result<()>;

    fn all_categories(&self) -> Result<Vec<Category>>;
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn all_ratings(&self) -> Result<Vec<Rating>>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;

    fn delete_bbox_subscription(&mut self, _: &str) -> Result<()>;
}
