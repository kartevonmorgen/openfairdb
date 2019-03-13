// Low-level database access traits.
// Each repository is responsible for a single entity and
// its relationships. Related entities are only referenced
// by their id and never modified or loaded by another
// repository.

use super::{entities::*, error::RepoError, util::time::Timestamp};

type Result<T> = std::result::Result<T, RepoError>;

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
            debug_assert!(rating.archived.is_none());
            let comments = self.load_comments_of_rating(&rating.id)?;
            results.push((rating, comments));
        }
        Ok(results)
    }

    fn archive_comments(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;
    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        archived: Timestamp,
    ) -> Result<usize>;
    fn archive_comments_of_entries(&self, entry_ids: &[&str], archived: Timestamp)
        -> Result<usize>;
}

pub trait RatingRepository {
    fn create_rating(&self, rating: Rating) -> Result<()>;

    // Only unarchived ratings without comments
    fn load_rating(&self, id: &str) -> Result<Rating>;
    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>>;
    fn load_ratings_of_entry(&self, entry_id: &str) -> Result<Vec<Rating>>;

    fn archive_ratings(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;
    fn archive_ratings_of_entries(&self, entry_ids: &[&str], archived: Timestamp) -> Result<usize>;

    fn load_entry_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>>;
}

pub trait EmailTokenCredentialsRepository {
    fn replace_email_token_credentials(
        &self,
        email_token_credentials: EmailTokenCredentials,
    ) -> Result<EmailTokenCredentials>;

    fn consume_email_token_credentials(
        &self,
        email_or_username: &str,
        token: &EmailToken,
    ) -> Result<EmailTokenCredentials>;

    fn discard_expired_email_token_credentials(&self, expired_before: Timestamp) -> Result<usize>;
}
