use crate::core::prelude::*;

pub fn archive_events<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    debug!("Archiving events {:?}", ids);
    let archived = Timestamp::now();
    db.archive_events(ids, archived)?;
    Ok(())
}
