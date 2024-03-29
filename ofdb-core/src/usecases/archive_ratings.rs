use super::prelude::*;

pub fn archive_ratings<R>(repo: &R, user_email: EmailAddress, ids: &[&str]) -> Result<usize>
where
    R: RatingRepository + CommentRepository + UserRepo,
{
    log::debug!("Archiving ratings {:?}", ids);
    // TODO: Pass an authentication token with user id and role to
    // check if the user is authorized to perform this use case
    let user = repo.try_get_user_by_email(&user_email)?;
    if let Some(user) = user {
        if user.role >= Role::Scout {
            let archived = Activity::now(Some(user_email));
            repo.archive_comments_of_ratings(ids, &archived)?;
            return Ok(repo.archive_ratings(ids, &archived)?);
        }
    }
    Err(Error::Forbidden)
}
