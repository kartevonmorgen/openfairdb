use super::*;

#[post("/ratings", format = "application/json", data = "<u>")]
pub fn post_rating(mut db: DbConn, u: Json<usecases::RateEntry>) -> Result<()> {
    let u = u.into_inner();
    let e_id = u.entry.clone();
    usecases::rate_entry(&mut *db, u)?;
    super::super::calculate_rating_for_entry(&*db, &e_id)?;
    Ok(Json(()))
}

#[get("/ratings/<ids>")]
pub fn get_rating(db: DbConn, ids: String) -> Result<Vec<json::Rating>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let mut ids = util::extract_ids(&ids);
    let ratings = usecases::get_ratings(&*db, &ids)?;
    // Retain only those ids that have actually been found
    debug_assert!(ratings.len() <= ids.len());
    ids.retain(|id| ratings.iter().any(|r| &r.id == id));
    debug_assert!(ratings.len() == ids.len());
    let comments = usecases::get_comments_by_rating_ids(&*db, &ids)?;
    let result = ratings
        .into_iter()
        .map(|x| json::Rating {
            id: x.id.clone(),
            created: x.created,
            title: x.title,
            value: x.value,
            context: x.context,
            source: x.source.unwrap_or_else(|| "".into()),
            comments: comments
                .get(&x.id)
                .cloned()
                .unwrap_or_else(|| vec![])
                .into_iter()
                .map(|c| json::Comment {
                    id: c.id.clone(),
                    created: c.created,
                    text: c.text,
                })
                .collect(),
        })
        .collect();
    Ok(Json(result))
}
