use crate::core::prelude::*;

use chrono::{Duration, Utc};

pub fn refresh_email_token_credentials<D: Db>(
    db: &D,
    username: String,
    email: String,
) -> Result<EmailTokenCredentials> {
    let token = EmailToken {
        email,
        nonce: Nonce::new(),
    };
    let credentials = EmailTokenCredentials {
        expires_at: Timestamp::from(Utc::now() + Duration::days(1)),
        username,
        token,
    };
    Ok(db.replace_email_token_credentials(credentials)?)
}

pub fn consume_email_token_credentials<D: Db>(
    db: &D,
    email_or_username: &str,
    token: &EmailToken,
) -> Result<EmailTokenCredentials> {
    let credentials = db.consume_email_token_credentials(email_or_username, token)?;
    if credentials.expires_at < Timestamp::now() {
        return Err(Error::Parameter(ParameterError::TokenExpired));
    }
    Ok(credentials)
}

pub fn discard_expired_email_token_credentials<D: Db>(db: &D) -> Result<usize> {
    let expired_before = Timestamp::now();
    Ok(db.discard_expired_email_token_credentials(expired_before)?)
}
