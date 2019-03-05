use crate::core::prelude::*;

use chrono::Utc;

pub fn archive_comments<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    debug!("Archiving comments {:?}", ids);
    let archived = Utc::now().timestamp() as u64;
    db.archive_comments(ids, archived)?;
    Ok(())
}
