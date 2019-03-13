use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowId(i64);

impl From<RowId> for i64 {
    fn from(from: RowId) -> Self {
        from.0
    }
}

impl From<i64> for RowId {
    fn from(from: i64) -> Self {
        Self(from)
    }
}

impl fmt::Display for RowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
