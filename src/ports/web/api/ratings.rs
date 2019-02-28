use super::*;

use crate::infrastructure::flows::prelude as flows;

#[post("/ratings", format = "application/json", data = "<data>")]
pub fn post_rating(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    data: Json<usecases::RateEntry>,
) -> Result<()> {
    let _ = flows::add_rating(&connections, &mut search_engine, data.into_inner())?;
    Ok(Json(()))
}

#[get("/ratings/<ids>")]
pub fn get_rating(db: sqlite::Connections, ids: String) -> Result<Vec<json::Rating>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let mut ids = util::extract_ids(&ids);
    let (ratings, comments) = {
        let db = db.shared()?;
        let ratings = db.get_ratings(&ids)?;
        // Retain only those ids that have actually been found
        debug_assert!(ratings.len() <= ids.len());
        ids.retain(|id| ratings.iter().any(|r| &r.id == id));
        debug_assert!(ratings.len() == ids.len());
        let comments = usecases::get_comments_by_rating_ids(&*db, &ids)?;
        (ratings, comments)
    };
    let result = ratings
        .into_iter()
        .map(|x| json::Rating {
            id: x.id.clone(),
            created: x.created,
            title: x.title,
            value: x.value.into(),
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
