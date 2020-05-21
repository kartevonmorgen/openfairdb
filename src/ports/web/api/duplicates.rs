use super::Result;
use crate::{
    core::{
        prelude::*,
        usecases::{self, DuplicateType},
        util,
    },
    infrastructure::db::sqlite,
};
use rocket_contrib::json::Json;

#[post("/duplicates/check-place", data = "<body>")]
pub fn post_duplicates(
    db: sqlite::Connections,
    body: Json<usecases::NewPlace>,
) -> Result<Vec<(String, DuplicateType)>> {
    let new_place = body.into_inner();
    let possible_duplicate_entries = {
        // TODO:
        // Calculate an area of interest (bounding box)
        // to reduce the amount of analyzed entries.
        let db = db.shared()?;
        db.all_places()?
    };
    let results =
        usecases::find_duplicate_of_unregistered_place(&new_place, &possible_duplicate_entries);
    Ok(Json(
        results
            .into_iter()
            .map(|(id, dup)| (id.to_string(), dup))
            .collect(),
    ))
}

#[get("/duplicates/<ids>")]
pub fn get_duplicates(
    db: sqlite::Connections,
    ids: String,
) -> Result<Vec<(String, String, DuplicateType)>> {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let (entries, possible_duplicate_entries) = {
        // TODO:
        // Calculate an area of interest (bounding box)
        // to reduce the amount of analyzed entries.
        let db = db.shared()?;
        (db.get_places(&ids)?, db.all_places()?)
    };
    let results = usecases::find_duplicates(&entries, &possible_duplicate_entries);
    Ok(Json(
        results
            .into_iter()
            .map(|(id1, id2, dup)| (id1.to_string(), id2.to_string(), dup))
            .collect(),
    ))
}
