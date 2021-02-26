use ofdb_boundary::PendingClearanceForPlace;
use ofdb_entities::place::{PlaceHistory, PlaceRevision};

pub const API_ROOT: &str = "/api";

#[derive(Clone, Debug)]
pub struct PlaceClearance {
    pub pending: PendingClearanceForPlace,
    pub history: Option<PlaceHistory>,
}

impl PlaceClearance {
    pub fn last_cleared_rev_nr(&self) -> Option<u64> {
        self.pending.last_cleared_revision
    }
    pub fn current_rev(&self) -> Option<&PlaceRevision> {
        self.history
            .as_ref()
            .and_then(|h| h.revisions.iter().nth(0).map(|(rev, _)| rev))
    }
    pub fn last_cleared_rev(&self) -> Option<&PlaceRevision> {
        let nr = self.last_cleared_rev_nr()?;
        let rev = self.history.as_ref()?.revisions.iter().find(|&x| {
            let r: u64 = x.0.revision.into();
            r == nr
        })?;
        Some(&rev.0)
    }
    pub fn overview_title(&self) -> &str {
        if let Some(rev) = self.last_cleared_rev() {
            &rev.title
        } else {
            if let Some(r) = self.current_rev() {
                &r.title
            } else {
                &self.pending.place_id
            }
        }
    }
}
