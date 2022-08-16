use crate::{id::*, time::*};

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub id          : Id,
    pub rating_id   : Id,
    // TODO: Convert time stamps from second to millisecond precision?
    pub created_at  : Timestamp,
    pub archived_at : Option<Timestamp>,
    pub text        : String,
}
