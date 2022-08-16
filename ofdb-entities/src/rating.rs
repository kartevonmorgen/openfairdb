use crate::{id::*, time::*};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

impl RatingContext {
    // The number of different contexts, i.e. the number of enum variants
    pub const fn total_count() -> u8 {
        6
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct RatingValue(i8);

impl RatingValue {
    pub fn new<I: Into<i8>>(val: I) -> Self {
        let new = Self(val.into());
        debug_assert!(new.is_valid());
        new
    }

    pub const fn min() -> Self {
        Self(-1)
    }

    pub const fn max() -> Self {
        Self(2)
    }

    pub fn clamp(self) -> Self {
        Self(self.0.max(Self::min().0).min(Self::max().0))
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<i8> for RatingValue {
    fn from(from: i8) -> Self {
        Self(from)
    }
}

impl From<RatingValue> for i8 {
    fn from(from: RatingValue) -> Self {
        from.0
    }
}

impl From<RatingValue> for f64 {
    fn from(from: RatingValue) -> Self {
        f64::from(from.0)
    }
}

impl std::ops::Add for AvgRatingValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for AvgRatingValue {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl std::ops::Div<f64> for AvgRatingValue {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs)
    }
}

impl std::ops::DivAssign<f64> for AvgRatingValue {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct AvgRatingValue(f64);

impl AvgRatingValue {
    pub const fn min() -> Self {
        Self(-1.0)
    }

    pub const fn max() -> Self {
        Self(2.0)
    }

    pub fn clamp(self) -> Self {
        Self(self.0.max(Self::min().0).min(Self::max().0))
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<f64> for AvgRatingValue {
    fn from(from: f64) -> Self {
        Self(from)
    }
}

impl From<AvgRatingValue> for f64 {
    fn from(from: AvgRatingValue) -> Self {
        from.0
    }
}

impl From<RatingValue> for AvgRatingValue {
    fn from(from: RatingValue) -> Self {
        f64::from(i8::from(from)).into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct AvgRatingValueBuilder {
    acc: i64,
    cnt: usize,
}

impl AvgRatingValueBuilder {
    fn add(&mut self, val: RatingValue) {
        debug_assert!(val.is_valid());
        self.acc += i64::from(val.0);
        self.cnt += 1;
    }

    pub fn build(self) -> AvgRatingValue {
        if self.cnt > 0 {
            AvgRatingValue::from(self.acc as f64 / self.cnt as f64).clamp()
        } else {
            Default::default()
        }
    }
}

impl std::ops::AddAssign<RatingValue> for AvgRatingValueBuilder {
    fn add_assign(&mut self, rhs: RatingValue) {
        self.add(rhs);
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct AvgRatings {
    pub diversity: AvgRatingValue,
    pub fairness: AvgRatingValue,
    pub humanity: AvgRatingValue,
    pub renewable: AvgRatingValue,
    pub solidarity: AvgRatingValue,
    pub transparency: AvgRatingValue,
}

impl AvgRatings {
    pub fn total(&self) -> AvgRatingValue {
        ((self.diversity
            + self.fairness
            + self.humanity
            + self.renewable
            + self.solidarity
            + self.transparency)
            / f64::from(RatingContext::total_count()))
        .clamp()
    }
}

#[derive(Debug, Default, Clone)]
pub struct AvgRatingsBuilder {
    pub diversity: AvgRatingValueBuilder,
    pub fairness: AvgRatingValueBuilder,
    pub humanity: AvgRatingValueBuilder,
    pub renewable: AvgRatingValueBuilder,
    pub solidarity: AvgRatingValueBuilder,
    pub transparency: AvgRatingValueBuilder,
}

impl AvgRatingsBuilder {
    pub fn add(&mut self, ctx: RatingContext, val: RatingValue) {
        use RatingContext::*;
        match ctx {
            Diversity => self.diversity.add(val),
            Fairness => self.fairness.add(val),
            Humanity => self.humanity.add(val),
            Renewable => self.renewable.add(val),
            Solidarity => self.solidarity.add(val),
            Transparency => self.transparency.add(val),
        }
    }

    pub fn build(self) -> AvgRatings {
        AvgRatings {
            diversity: self.diversity.build(),
            fairness: self.fairness.build(),
            humanity: self.humanity.build(),
            renewable: self.renewable.build(),
            solidarity: self.solidarity.build(),
            transparency: self.transparency.build(),
        }
    }
}

impl std::ops::AddAssign<(RatingContext, RatingValue)> for AvgRatingsBuilder {
    fn add_assign(&mut self, rhs: (RatingContext, RatingValue)) {
        self.add(rhs.0, rhs.1);
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rating {
    pub id          : Id,
    pub place_id    : Id,
    // TODO: Convert time stamps from second to millisecond precision?
    pub created_at  : Timestamp,
    pub archived_at : Option<Timestamp>,
    pub title       : String,
    pub value       : RatingValue,
    pub context     : RatingContext,
    pub source      : Option<String>,
}
