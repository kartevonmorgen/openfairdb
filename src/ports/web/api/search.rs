use super::super::{sqlite::DbConn, util};
use crate::{
    adapters::json,
    core::{prelude::*, usecases, util::geo},
    infrastructure::error::AppError,
};

use rocket::{self, request::Form};
use rocket_contrib::json::Json;
use std::result;

#[derive(FromForm, Clone)]
pub struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    text: Option<String>,
    tags: Option<String>,
}

type Result<T> = result::Result<Json<T>, AppError>;

#[get("/search?<search..>")]
pub fn get_search(db: DbConn, search: Form<SearchQuery>) -> Result<json::SearchResponse> {
    let bbox = geo::extract_bbox(&search.bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let categories = match search.categories {
        Some(ref cat_str) => Some(util::extract_ids(&cat_str)),
        None => None,
    };

    let mut tags = vec![];

    if let Some(ref txt) = search.text {
        tags = util::extract_hash_tags(txt);
    }

    if let Some(ref tags_str) = search.tags {
        for t in util::extract_ids(tags_str) {
            tags.push(t);
        }
    }

    let text = match search.text {
        Some(ref txt) => util::remove_hash_tags(txt),
        None => "".into(),
    };

    let avg_ratings = match super::super::ENTRY_RATINGS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    let req = usecases::SearchRequest {
        bbox,
        categories,
        text,
        tags,
        entry_ratings: &*avg_ratings,
    };

    let (visible, invisible) = usecases::search(&*db, &req)?;

    let visible = visible
        .into_iter()
        .map(json::EntryIdWithCoordinates::from)
        .collect();

    let invisible = invisible
        .into_iter()
        .map(json::EntryIdWithCoordinates::from)
        .collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}
