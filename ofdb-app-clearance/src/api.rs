use ofdb_boundary::PendingClearanceForPlace;
use ofdb_entities::place::{PlaceHistory, PlaceRevision};

pub const API_ROOT: &str = "/api";

#[derive(Clone, Debug)]
pub struct PlaceClearance {
    pub pending: PendingClearanceForPlace,
    pub history: PlaceHistory,
    pub expanded: bool,
}

impl PlaceClearance {
    pub fn last_cleared_rev_nr(&self) -> Option<u64> {
        self.pending.last_cleared_revision
    }
    pub fn current_rev_nr(&self) -> u64 {
        self.current_rev().revision.into()
    }
    pub fn current_rev(&self) -> &PlaceRevision {
        &self.history.revisions[0].0
    }
    pub fn last_cleared_rev(&self) -> Option<&PlaceRevision> {
        let nr = self.last_cleared_rev_nr()?;
        let rev = self.history.revisions.iter().find(|&x| {
            let r: u64 = x.0.revision.into();
            r == nr
        })?;
        Some(&rev.0)
    }
    pub fn overview_title(&self) -> &str {
        if let Some(rev) = self.last_cleared_rev() {
            &rev.title
        } else {
            &self.current_rev().title
        }
    }
}
