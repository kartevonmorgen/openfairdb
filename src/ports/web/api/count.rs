use super::*;

#[get("/count/entries")]
pub fn get_count_entries(db: DbConn) -> Result<usize> {
    let entries = db.all_entries()?;
    Ok(Json(entries.len()))
}

#[get("/count/tags")]
pub fn get_count_tags(db: DbConn) -> Result<usize> {
    Ok(Json(db.all_tags()?.len()))
}
