use crate::{id::Id, revision::Revision, time::TimestampMs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingAuthorizationForPlace {
    pub place_id: Id,
    pub created_at: TimestampMs,
    pub last_authorized_revision: Option<Revision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationForPlace {
    pub place_id: Id,
    pub authorized_revision: Option<Revision>,
}
