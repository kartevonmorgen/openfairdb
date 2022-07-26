use super::prelude::*;

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
