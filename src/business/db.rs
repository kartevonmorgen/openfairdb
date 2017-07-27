use super::error::RepoError;
use std::result;
use entities::*;

type Result<T> = result::Result<T, RepoError>;

pub trait Repo<T> {
    fn get(&self, &str) -> Result<T>;
    fn all(&self) -> Result<Vec<T>>;
    fn create(&mut self, &T) -> Result<()>;
    fn update(&mut self, &T) -> Result<()>;
}

pub trait Db {
    fn create_entry(&mut self, &Entry) -> Result<()>;
    fn create_tag(&mut self, &Tag) -> Result<()>;
    fn create_triple(&mut self, &Triple) -> Result<()>;
    fn create_user(&mut self, &User) -> Result<()>;
    fn create_comment(&mut self, &Comment) -> Result<()>;
    fn create_rating(&mut self, &Rating) -> Result<()>;
    fn create_bbox_subscription(&mut self, &BboxSubscription) -> Result<()>;

    fn get_entry(&self, &str) -> Result<Entry>;
    fn get_user(&self, &str) -> Result<User>;

    fn all_entries(&self) -> Result<Vec<Entry>>;
    fn all_categories(&self) -> Result<Vec<Category>>;
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn all_triples(&self) -> Result<Vec<Triple>>;
    fn all_ratings(&self) -> Result<Vec<Rating>>;
    fn all_comments(&self) -> Result<Vec<Comment>>;
    fn all_users(&self) -> Result<Vec<User>>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;

    fn update_entry(&mut self, &Entry) -> Result<()>;
    fn confirm_email_address(&mut self, &str) -> Result<()>;

    fn delete_triple(&mut self, &Triple) -> Result<()>;
    fn delete_bbox_subscription(&mut self, &str) -> Result<()>;
}
