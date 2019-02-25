use super::*;

#[get("/count/entries")]
pub fn get_count_entries(db: DbConn) -> Result<usize> {
    Ok(Json(db.read_only()?.count_entries()?))
}

#[get("/count/tags")]
pub fn get_count_tags(db: DbConn) -> Result<usize> {
    Ok(Json(db.read_only()?.count_tags()?))
}
