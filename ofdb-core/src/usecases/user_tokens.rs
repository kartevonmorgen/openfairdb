use super::prelude::*;
use time::Duration;

pub fn refresh_user_token<R: UserTokenRepo>(repo: &R, email: String) -> Result<EmailNonce> {
    let email_nonce = EmailNonce {
        email,
        nonce: Nonce::new(),
    };
    let token = UserToken {
        email_nonce,
        expires_at: Timestamp::now() + Duration::days(1),
    };
    Ok(repo.replace_user_token(token)?)
}

pub fn consume_user_token<R: UserTokenRepo>(
    repo: &R,
    email_nonce: &EmailNonce,
) -> Result<UserToken> {
    let token = repo.consume_user_token(email_nonce)?;
    debug_assert_eq!(email_nonce, &token.email_nonce);
    if token.expires_at < Timestamp::now() {
        return Err(Error::TokenExpired);
    }
    Ok(token)
}

pub fn delete_expired_user_tokens<R: UserTokenRepo>(repo: &R) -> Result<usize> {
    let expired_before = Timestamp::now();
    Ok(repo.delete_expired_user_tokens(expired_before)?)
}
