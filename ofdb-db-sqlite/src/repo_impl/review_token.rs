use super::*;

impl ReviewTokenRepo for DbReadWrite<'_> {
    fn add_review_token(&self, review_token: &ReviewToken) -> Result<()> {
        add_review_token(&mut self.conn.borrow_mut(), review_token)
    }
    fn consume_review_token(&self, review_nonce: &ReviewNonce) -> Result<ReviewToken> {
        consume_review_token(&mut self.conn.borrow_mut(), review_nonce)
    }
    fn delete_expired_review_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        delete_expired_review_tokens(&mut self.conn.borrow_mut(), expired_before)
    }
}

impl ReviewTokenRepo for DbReadOnly<'_> {
    fn add_review_token(&self, _review_token: &ReviewToken) -> Result<()> {
        unreachable!();
    }
    fn consume_review_token(&self, _review_nonce: &ReviewNonce) -> Result<ReviewToken> {
        unreachable!();
    }
    fn delete_expired_review_tokens(&self, _expired_before: Timestamp) -> Result<usize> {
        unreachable!();
    }
}

impl ReviewTokenRepo for DbConnection<'_> {
    fn add_review_token(&self, review_token: &ReviewToken) -> Result<()> {
        add_review_token(&mut self.conn.borrow_mut(), review_token)
    }
    fn consume_review_token(&self, review_nonce: &ReviewNonce) -> Result<ReviewToken> {
        consume_review_token(&mut self.conn.borrow_mut(), review_nonce)
    }
    fn delete_expired_review_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        delete_expired_review_tokens(&mut self.conn.borrow_mut(), expired_before)
    }
}

fn add_review_token(conn: &mut SqliteConnection, token: &ReviewToken) -> Result<()> {
    let ReviewToken {
        expires_at,
        review_nonce,
    } = token;
    let place_rowid = resolve_place_rowid(conn, &review_nonce.place_id)?;
    let revision = RevisionValue::from(review_nonce.place_revision) as i64;
    let nonce = review_nonce.nonce.to_string();
    let expires_at = expires_at.as_millis();
    let new_review_token = models::NewReviewToken {
        nonce,
        expires_at,
        revision,
        place_rowid,
    };
    let _count = diesel::insert_into(schema::review_tokens::table)
        .values(&new_review_token)
        .execute(conn)
        .map_err(from_diesel_err)?;
    debug_assert_eq!(1, _count);
    Ok(())
}

fn consume_review_token(
    conn: &mut SqliteConnection,
    review_nonce: &ReviewNonce,
) -> Result<ReviewToken> {
    use schema::review_tokens::dsl;
    let token = get_review_token_by_place_id(conn, &review_nonce.place_id)?;

    let target = dsl::review_tokens.filter(dsl::nonce.eq(review_nonce.nonce.to_string()));

    if diesel::delete(target)
        .execute(conn)
        .map_err(from_diesel_err)?
        == 0
    {
        return Err(repo::Error::NotFound);
    }
    debug_assert_eq!(review_nonce, &token.review_nonce);
    Ok(token)
}

fn delete_expired_review_tokens(
    conn: &mut SqliteConnection,
    expired_before: Timestamp,
) -> Result<usize> {
    use schema::review_tokens::dsl;
    diesel::delete(dsl::review_tokens.filter(dsl::expires_at.lt(expired_before.as_millis())))
        .execute(conn)
        .map_err(from_diesel_err)
}

fn get_review_token_by_place_id(conn: &mut SqliteConnection, place_id: &Id) -> Result<ReviewToken> {
    let place_rowid = resolve_place_rowid(conn, place_id)?;
    use schema::{place::dsl as p_dsl, review_tokens::dsl as r_dsl};
    Ok(r_dsl::review_tokens
        .inner_join(p_dsl::place)
        .select((p_dsl::id, r_dsl::revision, r_dsl::expires_at, r_dsl::nonce))
        .filter(p_dsl::rowid.eq(place_rowid))
        .first::<models::ReviewTokenEntity>(conn)
        .map_err(from_diesel_err)?
        .into())
}
