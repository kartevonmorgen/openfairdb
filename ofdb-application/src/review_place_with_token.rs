use super::*;

pub fn review_place_with_review_nonce(
    connections: &sqlite::Connections,
    review_nonce: ReviewNonce,
    status: ReviewStatus,
) -> Result<()> {
    // The token should be consumed only once, even if the
    // following transaction for reviewing the place fails!
    let token = connections.exclusive()?.transaction(|conn| {
        usecases::consume_review_token(conn, &review_nonce).map_err(|err| {
            log::warn!(
                "Missing or invalid token to review place ({}) for revision '{}': {}",
                review_nonce.place_id,
                RevisionValue::from(review_nonce.place_revision),
                err
            );
            err
        })
    })?;

    // The consumed nonce must match the request parameters
    debug_assert!(token.review_nonce == review_nonce);

    // Verify and review the place. This is done in a second transaction
    // to avoid keeping the nonce or using a nonce twice when this
    // transactions fails. Failures are expected if this place revision
    // has already been reviewed by someone else.
    connections.exclusive()?.transaction(|conn| {
        usecases::review_place_with_nonce(conn, token.review_nonce, status).map_err(|err| {
            warn!(
                "Failed to review place ({}) by token: {}",
                review_nonce.place_id, err
            );
            err
        })
    })?;
    Ok(())
}
