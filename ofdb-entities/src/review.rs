use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::*;
use strum::{EnumCount, EnumIter};

use crate::{activity::*, revision::*};

pub type ReviewStatusPrimitive = i16;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive, EnumIter, EnumCount)]
pub enum ReviewStatus {
    Rejected  = -1,
    Archived  =  0,
    Created   =  1,
    Confirmed =  2,
}

impl ReviewStatus {
    pub fn exists(self) -> bool {
        self >= Self::Created
    }

    pub const fn default() -> Self {
        Self::Created
    }

    pub fn try_from(from: ReviewStatusPrimitive) -> Option<Self> {
        Self::from_i16(from)
    }
}

impl From<ReviewStatus> for ReviewStatusPrimitive {
    fn from(from: ReviewStatus) -> Self {
        from.to_i16().unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewStatusLog {
    pub revision: Revision,
    pub activity: ActivityLog,
    pub status: ReviewStatus,
}
