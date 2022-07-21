// Low-level database access traits.
// Each repository is responsible for a single entity and
// its relationships. Related entities are only referenced
// by their id and never modified or loaded by another
// repository.

use crate::entities::*;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The requested object could not be found")]
    NotFound,
    #[error("The object already exists")]
    AlreadyExists,
    #[error("The version of the object is invalid")]
    InvalidVersion,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

pub trait CommentRepository {
    fn create_comment(&self, _: Comment) -> Result<()>;

    // Only unarchived comments
    fn load_comment(&self, id: &str) -> Result<Comment>;
    fn load_comments(&self, id: &[&str]) -> Result<Vec<Comment>>;
    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>>;

    // Only unarchived comments (even if the rating has already been archived)
    fn zip_ratings_with_comments(
        &self,
        ratings: Vec<Rating>,
    ) -> Result<Vec<(Rating, Vec<Comment>)>> {
        let mut results = Vec::with_capacity(ratings.len());
        for rating in ratings {
            debug_assert!(rating.archived_at.is_none());
            let comments = self.load_comments_of_rating(rating.id.as_ref())?;
            results.push((rating, comments));
        }
        Ok(results)
    }

    fn archive_comments(&self, ids: &[&str], activity: &Activity) -> Result<usize>;
    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        activity: &Activity,
    ) -> Result<usize>;
    fn archive_comments_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize>;
}

pub trait RatingRepository {
    fn create_rating(&self, rating: Rating) -> Result<()>;

    // Only unarchived ratings without comments
    fn load_rating(&self, id: &str) -> Result<Rating>;
    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>>;
    fn load_ratings_of_place(&self, place_id: &str) -> Result<Vec<Rating>>;

    fn archive_ratings(&self, ids: &[&str], activity: &Activity) -> Result<usize>;
    fn archive_ratings_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize>;

    fn load_place_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>>;
}

pub trait UserTokenRepo {
    fn replace_user_token(&self, user_token: UserToken) -> Result<EmailNonce>;

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken>;

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize>;

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken>;
}
