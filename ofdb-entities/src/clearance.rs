use crate::{id::Id, revision::Revision, time::TimestampMs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingClearanceForPlace {
    pub place_id: Id,
    pub created_at: TimestampMs,
    pub last_cleared_revision: Option<Revision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearanceForPlace {
    pub place_id: Id,
    pub cleared_revision: Option<Revision>,
}
