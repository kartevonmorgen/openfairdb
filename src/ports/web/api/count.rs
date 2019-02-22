use super::*;

#[get("/count/entries")]
pub fn get_count_entries(db: DbConn) -> Result<usize> {
    // TODO: Replace with count query
    let entries = db.all_entries()?;
    Ok(Json(entries.len()))
}

#[get("/count/tags")]
pub fn get_count_tags(db: DbConn) -> Result<usize> {
    // TODO: Replace with count query
    Ok(Json(db.all_tags()?.len()))
}
