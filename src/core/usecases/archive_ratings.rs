use crate::core::prelude::*;

use chrono::Utc;

pub fn archive_ratings<D: Db>(db: &D, ids: &[&str]) -> Result<Vec<String>> {
    debug!("Archiving ratings {:?}", ids);
    let archived = Utc::now().timestamp() as u64;
    let entry_ids = db.archive_ratings(ids, archived)?;
    Ok(entry_ids)
}
