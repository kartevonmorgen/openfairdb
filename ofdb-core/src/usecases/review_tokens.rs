use super::prelude::*;

pub fn consume_review_token<R: ReviewTokenRepo>(
    repo: &R,
    review_nonce: &ReviewNonce,
    current_time: Timestamp,
) -> Result<ReviewToken> {
    let token = repo.consume_review_token(review_nonce)?;
    debug_assert_eq!(review_nonce, &token.review_nonce);
    if token.expires_at < current_time {
        return Err(Error::TokenExpired);
    }
    Ok(token)
}

pub fn delete_expired_review_tokens<R: ReviewTokenRepo>(
    repo: &R,
    expired_before: Timestamp,
) -> Result<usize> {
    Ok(repo.delete_expired_review_tokens(expired_before)?)
}
