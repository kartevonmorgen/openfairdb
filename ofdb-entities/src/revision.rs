#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Revision(u64);

impl Revision {
    pub const fn initial() -> Self {
        Self(0)
    }

    pub fn is_initial(self) -> bool {
        self == Self::initial()
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl From<Revision> for u64 {
    fn from(from: Revision) -> Self {
        from.0
    }
}

impl From<u64> for Revision {
    fn from(from: u64) -> Self {
        Self(from)
    }
}
