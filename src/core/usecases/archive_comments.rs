use crate::core::prelude::*;

pub fn archive_comments<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    info!("Archiving {} comments", ids.len());
    let archived = Timestamp::now();
    db.archive_comments(ids, archived)?;
    Ok(())
}
