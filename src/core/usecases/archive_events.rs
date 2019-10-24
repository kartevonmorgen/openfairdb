use crate::core::prelude::*;

pub fn archive_events<D: Db>(db: &D, uids: &[&str]) -> Result<()> {
    debug!("Archiving events {:?}", uids);
    let archived = Timestamp::now();
    db.archive_events(uids, archived)?;
    Ok(())
}
