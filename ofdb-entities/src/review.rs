use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::*;
use strum::{EnumCount, EnumIter, EnumString};
use thiserror::Error;

use crate::{activity::*, revision::*};

pub type ReviewStatusPrimitive = i16;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive, EnumIter, EnumCount, EnumString)]
#[strum(ascii_case_insensitive)]
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
}

#[derive(Debug, Error)]
#[error("Invalid review status primitive: {0}")]
pub struct InvalidReviewStatusPrimitive(ReviewStatusPrimitive);

impl TryFrom<i16> for ReviewStatus {
    type Error = InvalidReviewStatusPrimitive;
    fn try_from(from: ReviewStatusPrimitive) -> Result<Self, Self::Error> {
        Self::from_i16(from).ok_or(InvalidReviewStatusPrimitive(from))
    }
}

impl From<ReviewStatus> for ReviewStatusPrimitive {
    fn from(from: ReviewStatus) -> Self {
        from.to_i16().expect("Review status primitive")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewStatusLog {
    pub revision: Revision,
    pub activity: ActivityLog,
    pub status: ReviewStatus,
}
