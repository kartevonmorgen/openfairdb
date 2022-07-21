use super::prelude::*;
use time::Duration;

pub fn refresh_user_token<D: Db>(db: &D, email: String) -> Result<EmailNonce> {
    let email_nonce = EmailNonce {
        email,
        nonce: Nonce::new(),
    };
    let token = UserToken {
        email_nonce,
        expires_at: Timestamp::now() + Duration::days(1),
    };
    Ok(db.replace_user_token(token)?)
}

pub fn consume_user_token<D: Db>(db: &D, email_nonce: &EmailNonce) -> Result<UserToken> {
    let token = db.consume_user_token(email_nonce)?;
    debug_assert_eq!(email_nonce, &token.email_nonce);
    if token.expires_at < Timestamp::now() {
        return Err(Error::TokenExpired);
    }
    Ok(token)
}

pub fn delete_expired_user_tokens<D: Db>(db: &D) -> Result<usize> {
    let expired_before = Timestamp::now();
    Ok(db.delete_expired_user_tokens(expired_before)?)
}
