pub type RevisionValue = u64;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Revision(RevisionValue);

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

impl From<Revision> for RevisionValue {
    fn from(from: Revision) -> Self {
        from.0
    }
}

impl From<RevisionValue> for Revision {
    fn from(from: RevisionValue) -> Self {
        Self(from)
    }
}
