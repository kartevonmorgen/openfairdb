use super::*;

impl UserTokenRepo for Connection<'_> {
    fn replace_user_token(&self, token: UserToken) -> Result<EmailNonce> {
        use schema::user_tokens::dsl;
        let user_id = resolve_user_created_by_email(self, &token.email_nonce.email)?;
        let model = models::NewUserToken {
            user_id,
            nonce: token.email_nonce.nonce.to_string(),
            expires_at: token.expires_at.as_millis(),
        };
        // Insert...
        if diesel::insert_into(schema::user_tokens::table)
            .values(&model)
            .execute(self.deref())
            .map_err(from_diesel_err)?
            == 0
        {
            // ...or update
            let _count = diesel::update(schema::user_tokens::table)
                .filter(dsl::user_id.eq(model.user_id))
                .set(&model)
                .execute(self.deref())
                .map_err(from_diesel_err)?;
            debug_assert_eq!(1, _count);
        }
        Ok(token.email_nonce)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        use schema::{user_tokens::dsl as t_dsl, users::dsl as u_dsl};
        let token = self.get_user_token_by_email(&email_nonce.email)?;
        let user_id_subselect = u_dsl::users
            .select(u_dsl::id)
            .filter(u_dsl::email.eq(&email_nonce.email));
        let target = t_dsl::user_tokens
            .filter(t_dsl::nonce.eq(email_nonce.nonce.to_string()))
            .filter(t_dsl::user_id.eq_any(user_id_subselect));
        if diesel::delete(target)
            .execute(self.deref())
            .map_err(from_diesel_err)?
            == 0
        {
            return Err(repo::Error::NotFound);
        }
        debug_assert_eq!(email_nonce, &token.email_nonce);
        Ok(token)
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        use schema::user_tokens::dsl;
        diesel::delete(dsl::user_tokens.filter(dsl::expires_at.lt(expired_before.as_millis())))
            .execute(self.deref())
            .map_err(from_diesel_err)
    }

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken> {
        use schema::{user_tokens::dsl as t_dsl, users::dsl as u_dsl};
        Ok(t_dsl::user_tokens
            .inner_join(u_dsl::users)
            .select((u_dsl::id, t_dsl::nonce, t_dsl::expires_at, u_dsl::email))
            .filter(u_dsl::email.eq(email))
            .first::<models::UserTokenEntity>(self.deref())
            .map_err(from_diesel_err)?
            .into())
    }
}
