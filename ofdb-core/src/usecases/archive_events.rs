use super::prelude::*;

pub fn archive_events<D: Db>(db: &D, ids: &[&str]) -> Result<usize> {
    log::debug!("Archiving events {:?}", ids);
    let archived = Timestamp::now();
    Ok(db.archive_events(ids, archived)?)
}
