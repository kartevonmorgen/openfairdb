use std::fmt;
use time::{OffsetDateTime, PrimitiveDateTime};

pub trait InnerTimestampConverter: Clone + Copy + PartialEq + Eq + PartialOrd + Ord {
    type Inner: Clone + Copy + PartialEq + Eq + PartialOrd + Ord;

    #[allow(clippy::wrong_self_convention)]
    fn into_inner(ts: OffsetDateTime) -> Self::Inner;

    fn from_inner(ts: Self::Inner) -> OffsetDateTime;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecondsTimestampConverter;

impl InnerTimestampConverter for SecondsTimestampConverter {
    type Inner = i64;

    fn into_inner(ts: OffsetDateTime) -> Self::Inner {
        ts.unix_timestamp()
    }

    fn from_inner(ts: Self::Inner) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(ts).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MillisecondsTimestampConverter;

impl InnerTimestampConverter for MillisecondsTimestampConverter {
    type Inner = i64;

    fn into_inner(ts: OffsetDateTime) -> Self::Inner {
        nanos_to_millis(ts.unix_timestamp_nanos())
    }

    fn from_inner(ts: Self::Inner) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp_nanos(millis_to_nanos(ts)).unwrap()
    }
}

fn nanos_to_millis(nanos: i128) -> i64 {
    (nanos / 1_000_000).try_into().unwrap()
}

fn millis_to_nanos(millis: i64) -> i128 {
    i128::from(millis) * 1_000_000
}

// A generic timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenericTimestamp<C: InnerTimestampConverter>(C::Inner);

impl<C: InnerTimestampConverter> GenericTimestamp<C> {
    pub fn now() -> Self {
        OffsetDateTime::now_utc().into()
    }

    pub fn from_inner(from: C::Inner) -> Self {
        Self(from)
    }

    pub fn into_inner(self) -> C::Inner {
        self.0
    }

    pub fn from_seconds(seconds: i64) -> Self {
        Self(C::into_inner(SecondsTimestampConverter::from_inner(
            seconds,
        )))
    }

    pub fn from_milliseconds(milliseconds: i64) -> Self {
        Self(C::into_inner(MillisecondsTimestampConverter::from_inner(
            milliseconds,
        )))
    }

    pub fn into_seconds(self) -> i64 {
        C::from_inner(self.0).unix_timestamp()
    }

    pub fn into_milliseconds(self) -> i64 {
        nanos_to_millis(C::from_inner(self.0).unix_timestamp_nanos())
    }
}

impl<C: InnerTimestampConverter> From<PrimitiveDateTime> for GenericTimestamp<C> {
    fn from(from: PrimitiveDateTime) -> Self {
        Self(C::into_inner(from.assume_utc()))
    }
}

impl<C: InnerTimestampConverter> From<GenericTimestamp<C>> for OffsetDateTime {
    fn from(from: GenericTimestamp<C>) -> Self {
        C::from_inner(from.0)
    }
}

impl<C: InnerTimestampConverter> From<OffsetDateTime> for GenericTimestamp<C> {
    fn from(from: OffsetDateTime) -> Self {
        Self(C::into_inner(from))
    }
}

impl<C: InnerTimestampConverter> fmt::Display for GenericTimestamp<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", OffsetDateTime::from(self.to_owned()))
    }
}

pub type Timestamp = GenericTimestamp<SecondsTimestampConverter>;

pub type TimestampMs = GenericTimestamp<MillisecondsTimestampConverter>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_from_into_inner() {
        let t1 = Timestamp::now();
        let i1 = t1.into_inner();
        let t2 = Timestamp::from_inner(i1);
        assert_eq!(t1, t2);
    }

    #[test]
    fn convert_from_into_inner_ms() {
        let t1 = TimestampMs::now();
        let i1 = t1.into_inner();
        let t2 = TimestampMs::from_inner(i1);
        assert_eq!(t1, t2);
    }
}
