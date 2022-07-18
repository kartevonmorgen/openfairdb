use std::{
    fmt,
    ops::{Add, Sub},
};
use time::{
    format_description::FormatItem, macros::format_description, Duration, OffsetDateTime,
    PrimitiveDateTime,
};

/// A primitive unix timestamp without time zone.
///
/// We assume that all timestamps must be specified in the UTC time zone.

// This is a temporary workaround because
// [`time`](::time) does not allow to convert [`OffsetDateTime`] to [`PrimitiveDateTime`]
// (see <https://github.com/time-rs/time/pull/458>)
// and [`PrimitiveDateTime`] has e.g. no `unix_timestamp` method.
// So we internally use [`OffsetDateTime`] but the semantic is like [`PrimitiveDateTime`].
#[derive(Debug, Copy, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Timestamp(time::OffsetDateTime);

const TIMESTAMP_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.format(TIMESTAMP_FORMAT).unwrap())
    }
}

impl Timestamp {
    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    pub fn from_secs(seconds: i64) -> Self {
        Self(time::OffsetDateTime::from_unix_timestamp(seconds).unwrap())
    }

    pub fn from_millis(milliseconds: i64) -> Self {
        let nanos = millis_to_nanos(milliseconds);
        Self(time::OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap())
    }

    pub fn as_secs(self) -> i64 {
        self.0.unix_timestamp()
    }

    pub fn as_millis(self) -> i64 {
        nanos_to_millis(self.0.unix_timestamp_nanos())
    }

    pub fn format(&self, fmt: &[FormatItem<'_>]) -> String {
        self.0.format(fmt).unwrap()
    }
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        self.0.checked_sub(duration).map(Self)
    }
}

fn nanos_to_millis(nanos: i128) -> i64 {
    (nanos / 1_000_000).try_into().unwrap()
}

fn millis_to_nanos(millis: i64) -> i128 {
    i128::from(millis) * 1_000_000
}

impl From<PrimitiveDateTime> for Timestamp {
    fn from(ts: PrimitiveDateTime) -> Self {
        Self(ts.assume_utc())
    }
}

impl Add<Duration> for Timestamp {
    type Output = Self;
    fn add(self, d: time::Duration) -> Self {
        Self(self.0.add(d))
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Self;
    fn sub(self, d: time::Duration) -> Self {
        Self(self.0.sub(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp() {
        let ts = Timestamp::from_millis(1_658_146_497_321);
        assert_eq!("2022-07-18 12:14:57.321", format!("{ts}"));
    }
}
