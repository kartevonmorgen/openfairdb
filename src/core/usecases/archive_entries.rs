use crate::core::prelude::*;

pub fn archive_entries<D: Db>(db: &D, ids: &[&str], email: Option<&str>) -> Result<()> {
    info!("Archiving {} entries", ids.len());
    let activity = Activity::now(email.map(Into::into));
    db.archive_comments_of_entries(ids, &activity)?;
    db.archive_ratings_of_entries(ids, &activity)?;
    db.archive_places(ids, &activity)?;
    Ok(())
}
