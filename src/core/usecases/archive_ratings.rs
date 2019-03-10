use crate::core::prelude::*;

pub fn archive_ratings<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    debug!("Archiving ratings {:?}", ids);
    let archived = Timestamp::now();
    db.archive_comments_of_ratings(ids, archived)?;
    db.archive_ratings(ids, archived)?;
    Ok(())
}
