use crate::{email::*, time::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub at: TimestampMs,
    pub by: Option<Email>,
}

impl Activity {
    pub fn now(by: Option<Email>) -> Self {
        Self {
            at: TimestampMs::now(),
            by,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityLog {
    pub activity: Activity,
    pub context: Option<String>,
    pub comment: Option<String>,
}
