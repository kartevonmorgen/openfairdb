use crate::core::prelude::*;

pub fn archive_entries<D: Db>(db: &D, ids: &[&str]) -> Result<()> {
    info!("Archiving {} entries", ids.len());
    let archived = Timestamp::now();
    db.archive_comments_of_entries(ids, archived)?;
    db.archive_ratings_of_entries(ids, archived)?;
    db.archive_entries(ids, archived)?;
    Ok(())
}
