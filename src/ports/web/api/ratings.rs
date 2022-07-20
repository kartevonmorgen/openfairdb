use super::*;
use crate::{core::util, infrastructure::flows::prelude as flows};

#[post("/ratings", format = "application/json", data = "<data>")]
pub fn post_rating(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    data: JsonResult<usecases::NewPlaceRating>,
) -> Result<()> {
    let _ = flows::create_rating(&connections, &mut search_engine, data?.into_inner())?;
    Ok(Json(()))
}

#[get("/ratings/<ids>")]
pub fn load_rating(db: sqlite::Connections, ids: String) -> Result<Vec<json::Rating>> {
    // TODO: RESTful API
    //   - Only lookup and return a single entity
    //   - Add a new action and method for getting multiple ids at once
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let ratings_with_comments = usecases::load_ratings_with_comments(&db.shared()?, &ids)?;
    let result = ratings_with_comments
        .into_iter()
        .map(|(r, cs)| {
            let comments = cs
                .into_iter()
                .map(|c| json::Comment {
                    id: c.id.clone().into(),
                    created: c.created_at.as_secs(),
                    text: c.text,
                })
                .collect();
            json::Rating {
                id: r.id.into(),
                created: r.created_at.as_secs(),
                title: r.title,
                value: r.value.into(),
                context: r.context.into(),
                source: r.source.unwrap_or_default(),
                comments,
            }
        })
        .collect();
    Ok(Json(result))
}
