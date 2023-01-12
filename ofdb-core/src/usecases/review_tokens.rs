use super::prelude::*;

pub fn consume_review_token<R: ReviewTokenRepo>(
    repo: &R,
    review_nonce: &ReviewNonce,
) -> Result<ReviewToken> {
    let token = repo.consume_review_token(review_nonce)?;
    debug_assert_eq!(review_nonce, &token.review_nonce);
    if token.expires_at < Timestamp::now() {
        return Err(Error::TokenExpired);
    }
    Ok(token)
}

pub fn delete_expired_review_tokens<R: ReviewTokenRepo>(repo: &R) -> Result<usize> {
    let expired_before = Timestamp::now();
    Ok(repo.delete_expired_review_tokens(expired_before)?)
}
