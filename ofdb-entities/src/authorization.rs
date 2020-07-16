use crate::{id::Id, review::ReviewStatus, revision::Revision, time::TimestampMs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedRevision {
    pub revision: Revision,
    pub review_status: Option<ReviewStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAuthorizationForPlace {
    pub place_id: Id,
    pub created_at: TimestampMs,
    pub last_authorized: Option<AuthorizedRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationForPlace {
    pub place_id: Id,
    pub created_at: TimestampMs,
    pub authorized: AuthorizedRevision,
}
