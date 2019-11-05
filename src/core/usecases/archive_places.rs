use crate::core::prelude::*;

pub fn archive_places<D: Db>(db: &D, ids: &[&str], email: Option<&str>) -> Result<()> {
    info!("Archiving {} places", ids.len());
    let activity = Activity::now(email.map(Into::into));
    db.archive_comments_of_places(ids, &activity)?;
    db.archive_ratings_of_places(ids, &activity)?;
    db.change_status_of_places(ids, Status::archived(), &activity)?;
    Ok(())
}
