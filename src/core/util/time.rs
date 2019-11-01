use chrono::{DateTime, NaiveDateTime, Utc};
use std::fmt;

// A timestamp in Utc with second precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn now() -> Self {
        Utc::now().into()
    }

    pub const fn into_inner(self) -> i64 {
        self.0
    }
}

impl From<Timestamp> for i64 {
    fn from(from: Timestamp) -> Self {
        from.0
    }
}

impl From<i64> for Timestamp {
    fn from(from: i64) -> Self {
        Self(from)
    }
}

impl From<NaiveDateTime> for Timestamp {
    fn from(from: NaiveDateTime) -> Self {
        Self(from.timestamp())
    }
}

impl From<Timestamp> for NaiveDateTime {
    fn from(from: Timestamp) -> Self {
        NaiveDateTime::from_timestamp(from.0, 0)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(from: DateTime<Utc>) -> Self {
        Self(from.timestamp())
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(from: Timestamp) -> Self {
        Self::from_utc(NaiveDateTime::from(from), Utc)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", DateTime::<Utc>::from(*self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_from_to_i64() {
        let t1 = Timestamp::now();
        let i1 = i64::from(t1);
        let t2 = Timestamp::from(i1);
        assert_eq!(t1, t2);
    }
}
