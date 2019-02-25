use super::*;

#[get("/count/entries")]
pub fn get_count_entries(db: sqlite::Connections) -> Result<usize> {
    Ok(Json(db.shared()?.count_entries()?))
}

#[get("/count/tags")]
pub fn get_count_tags(db: sqlite::Connections) -> Result<usize> {
    Ok(Json(db.shared()?.count_tags()?))
}
