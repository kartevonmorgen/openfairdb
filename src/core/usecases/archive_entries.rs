use crate::core::prelude::*;

use chrono::Utc;

pub fn archive_entries<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    debug!("Archiving entries {:?}", ids);
    let archived = Utc::now().timestamp() as u64;
    db.archive_entries(ids, archived)?;
    Ok(())
}
