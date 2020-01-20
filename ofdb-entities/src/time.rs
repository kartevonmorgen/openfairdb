use chrono::{DateTime, NaiveDateTime, Utc};
use std::fmt;

pub trait InnerTimestampConverter: Clone + Copy + PartialEq + Eq + PartialOrd + Ord {
    type Inner: Clone + Copy + PartialEq + Eq + PartialOrd + Ord;

    fn into_inner(ts: NaiveDateTime) -> Self::Inner;

    fn from_inner(ts: Self::Inner) -> NaiveDateTime;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SecondsTimestampConverter;

impl InnerTimestampConverter for SecondsTimestampConverter {
    type Inner = i64;

    fn into_inner(ts: NaiveDateTime) -> Self::Inner {
        ts.timestamp()
    }

    fn from_inner(ts: Self::Inner) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(ts, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MillisecondsTimestampConverter;

impl InnerTimestampConverter for MillisecondsTimestampConverter {
    type Inner = i64;

    fn into_inner(ts: NaiveDateTime) -> Self::Inner {
        ts.timestamp_millis()
    }

    fn from_inner(ts: Self::Inner) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(ts / 1000i64, (ts % 1000i64) as u32 * 1_000_000u32)
    }
}

// A generic timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenericTimestamp<C: InnerTimestampConverter>(C::Inner);

impl<C: InnerTimestampConverter> GenericTimestamp<C> {
    pub fn now() -> Self {
        Utc::now().into()
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
        C::from_inner(self.0).timestamp()
    }

    pub fn into_milliseconds(self) -> i64 {
        C::from_inner(self.0).timestamp_millis()
    }
}

impl<C: InnerTimestampConverter> From<NaiveDateTime> for GenericTimestamp<C> {
    fn from(from: NaiveDateTime) -> Self {
        Self(C::into_inner(from))
    }
}

impl<C: InnerTimestampConverter> From<GenericTimestamp<C>> for NaiveDateTime {
    fn from(from: GenericTimestamp<C>) -> Self {
        C::from_inner(from.0)
    }
}

impl<C: InnerTimestampConverter> From<DateTime<Utc>> for GenericTimestamp<C> {
    fn from(from: DateTime<Utc>) -> Self {
        Self(C::into_inner(from.naive_utc()))
    }
}

impl<C: InnerTimestampConverter> From<GenericTimestamp<C>> for DateTime<Utc> {
    fn from(from: GenericTimestamp<C>) -> Self {
        Self::from_utc(NaiveDateTime::from(from), Utc)
    }
}

impl<C: InnerTimestampConverter> fmt::Display for GenericTimestamp<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", DateTime::<Utc>::from(self.to_owned()))
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
