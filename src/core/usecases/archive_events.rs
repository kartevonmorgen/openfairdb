use crate::core::prelude::*;

use chrono::Utc;

pub fn archive_events<D: Db>(db: &mut D, ids: &[&str]) -> Result<()> {
    debug!("Archiving events {:?}", ids);
    let archived = Utc::now().timestamp() as u64;
    db.archive_events(ids, archived)?;
    Ok(())
}
