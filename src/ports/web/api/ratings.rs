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
    // TODO: RESTful API
    //   - Only lookup and return a single entity
    //   - Add a new action and method for getting multiple ids at once
    let ids = util::extract_ids(&ids);
    let ratings_with_comments = usecases::load_ratings_with_comments(&*db.shared()?, &ids)?;
    let result = ratings_with_comments
        .into_iter()
        .map(|(r, cs)| {
            let comments = cs
                .into_iter()
                .map(|c| json::Comment {
                    id: c.id.clone(),
                    created: c.created,
                    text: c.text,
                })
                .collect();
            json::Rating {
                id: r.id,
                created: r.created,
                title: r.title,
                value: r.value.into(),
                context: r.context,
                source: r.source.unwrap_or_default(),
                comments,
            }
        })
        .collect();
    Ok(Json(result))
}
