use super::*;

impl UserTokenRepo for DbReadWrite<'_> {
    fn replace_user_token(&self, user_token: UserToken) -> Result<EmailNonce> {
        replace_user_token(&mut self.conn.borrow_mut(), user_token)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        consume_user_token(&mut self.conn.borrow_mut(), email_nonce)
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        delete_expired_user_tokens(&mut self.conn.borrow_mut(), expired_before)
    }

    fn get_user_token_by_email(&self, email: &EmailAddress) -> Result<UserToken> {
        get_user_token_by_email(&mut self.conn.borrow_mut(), email)
    }
}

impl UserTokenRepo for DbConnection<'_> {
    fn replace_user_token(&self, user_token: UserToken) -> Result<EmailNonce> {
        replace_user_token(&mut self.conn.borrow_mut(), user_token)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        consume_user_token(&mut self.conn.borrow_mut(), email_nonce)
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        delete_expired_user_tokens(&mut self.conn.borrow_mut(), expired_before)
    }

    fn get_user_token_by_email(&self, email: &EmailAddress) -> Result<UserToken> {
        get_user_token_by_email(&mut self.conn.borrow_mut(), email)
    }
}

impl UserTokenRepo for DbReadOnly<'_> {
    fn replace_user_token(&self, _user_token: UserToken) -> Result<EmailNonce> {
        unreachable!();
    }

    fn consume_user_token(&self, _email_nonce: &EmailNonce) -> Result<UserToken> {
        unreachable!();
    }

    fn delete_expired_user_tokens(&self, _expired_before: Timestamp) -> Result<usize> {
        unreachable!();
    }

    fn get_user_token_by_email(&self, email: &EmailAddress) -> Result<UserToken> {
        get_user_token_by_email(&mut self.conn.borrow_mut(), email)
    }
}

fn replace_user_token(conn: &mut SqliteConnection, token: UserToken) -> Result<EmailNonce> {
    use schema::user_tokens::dsl;
    let user_id = resolve_user_created_by_email(conn, &token.email_nonce.email)?;
    let model = models::NewUserToken {
        user_id,
        nonce: token.email_nonce.nonce.to_string(),
        expires_at: token.expires_at.as_millis(),
    };
    // Insert...
    if diesel::insert_into(schema::user_tokens::table)
        .values(&model)
        .execute(conn)
        .map_err(from_diesel_err)?
        == 0
    {
        // ...or update
        let _count = diesel::update(schema::user_tokens::table)
            .filter(dsl::user_id.eq(model.user_id))
            .set(&model)
            .execute(conn)
            .map_err(from_diesel_err)?;
        debug_assert_eq!(1, _count);
    }
    Ok(token.email_nonce)
}

fn consume_user_token(conn: &mut SqliteConnection, email_nonce: &EmailNonce) -> Result<UserToken> {
    use schema::{user_tokens::dsl as t_dsl, users::dsl as u_dsl};
    let token = get_user_token_by_email(conn, &email_nonce.email)?;
    let user_id_subselect = u_dsl::users
        .select(u_dsl::id)
        .filter(u_dsl::email.eq(email_nonce.email.as_str()));
    let target = t_dsl::user_tokens
        .filter(t_dsl::nonce.eq(email_nonce.nonce.to_string()))
        .filter(t_dsl::user_id.eq_any(user_id_subselect));
    if diesel::delete(target)
        .execute(conn)
        .map_err(from_diesel_err)?
        == 0
    {
        return Err(repo::Error::NotFound);
    }
    debug_assert_eq!(email_nonce, &token.email_nonce);
    Ok(token)
}

fn delete_expired_user_tokens(
    conn: &mut SqliteConnection,
    expired_before: Timestamp,
) -> Result<usize> {
    use schema::user_tokens::dsl;
    diesel::delete(dsl::user_tokens.filter(dsl::expires_at.lt(expired_before.as_millis())))
        .execute(conn)
        .map_err(from_diesel_err)
}

fn get_user_token_by_email(conn: &mut SqliteConnection, email: &EmailAddress) -> Result<UserToken> {
    use schema::{user_tokens::dsl as t_dsl, users::dsl as u_dsl};
    Ok(t_dsl::user_tokens
        .inner_join(u_dsl::users)
        .select((u_dsl::id, t_dsl::nonce, t_dsl::expires_at, u_dsl::email))
        .filter(u_dsl::email.eq(email.as_str()))
        .first::<models::UserTokenEntity>(conn)
        .map_err(from_diesel_err)?
        .into())
}
