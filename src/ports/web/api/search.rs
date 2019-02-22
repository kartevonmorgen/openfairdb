use super::super::{sqlite::DbConn, tantivy::SearchEngine, util};
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
    limit: Option<usize>,
}

type Result<T> = result::Result<Json<T>, AppError>;

const MAX_RESULTS: usize = 100;

#[get("/search?<search..>")]
pub fn get_search(
    db: DbConn,
    search_engine: SearchEngine,
    search: Form<SearchQuery>,
) -> Result<json::SearchResponse> {
    let bbox = search
        .bbox
        .parse::<geo::MapBbox>()
        .map_err(|_| ParameterError::Bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let categories = search
        .categories
        .as_ref()
        .map(String::as_str)
        .map(util::extract_ids)
        .unwrap_or_else(|| vec![]);

    let mut tags = search
        .text
        .as_ref()
        .map(String::as_str)
        .map(util::extract_hash_tags)
        .unwrap_or_else(|| vec![]);
    if let Some(ref tags_str) = search.tags {
        for t in util::extract_ids(tags_str) {
            tags.push(t);
        }
    }

    let text = search
        .text
        .as_ref()
        .map(String::as_str)
        .map(util::remove_hash_tags)
        .and_then(|text| {
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        });

    let (visible, invisible) = {
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

        let search_limit = if let Some(limit) = search.limit {
            if limit > MAX_RESULTS {
                warn!(
                    "Reducing limit for search results from {} to {}",
                    limit, MAX_RESULTS
                );
                MAX_RESULTS
            } else {
                limit
            }
        } else {
            info!("Limiting search results to {}", MAX_RESULTS);
            MAX_RESULTS
        };

        let (visible, invisible) =
            usecases::search(&search_engine, &*db.read_only()?, req, Some(search_limit))?;

        let visible_len = visible.len();
        let visible: Vec<json::EntrySearchResult> = visible
            .into_iter()
            .take(visible_len.min(search_limit))
            .map(|e| {
                let avg_rating = avg_ratings.get(&e.id).cloned().unwrap_or(0.0);
                (e, avg_rating)
            })
            .map(Into::into)
            .collect();

        let invisible_len = invisible.len();
        let invisible: Vec<json::EntrySearchResult> = invisible
            .into_iter()
            .take(invisible_len.min(search_limit - search_limit.min(visible.len())))
            .map(|e| {
                let avg_rating = avg_ratings.get(&e.id).cloned().unwrap_or(0.0);
                (e, avg_rating)
            })
            .map(Into::into)
            .collect();

        (visible, invisible)
        // Implicitly release mutex on ENTRY_RATINGS before serializing response
    };

    Ok(Json(json::SearchResponse { visible, invisible }))
}
