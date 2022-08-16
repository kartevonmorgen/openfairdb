use crate::{geo::*, id::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BboxSubscription {
    pub id: Id,
    pub user_email: String,
    pub bbox: MapBbox,
}
