use super::super::util;
use crate::{
    adapters::json,
    core::{prelude::*, usecases, util::geo},
    infrastructure::{db::tantivy, error::AppError},
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
    search_engine: tantivy::SearchEngine,
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

    let req = usecases::SearchRequest {
        bbox,
        categories,
        text,
        tags,
    };

    let search_limit = if let Some(limit) = search.limit {
        if limit > MAX_RESULTS {
            info!(
                "Requested limit {} exceeds maximum limit {} for search results",
                limit, MAX_RESULTS
            );
            MAX_RESULTS
        } else if limit <= 0 {
            warn!("Invalid search limit: {}", limit);
            return Err(AppError::Business(Error::Parameter(
                ParameterError::InvalidLimit,
            )));
        } else {
            limit
        }
    } else {
        info!(
            "No limit requested - Using maximum limit {} for search results",
            MAX_RESULTS
        );
        MAX_RESULTS
    };

    let (visible, invisible) = usecases::search(&search_engine, req, search_limit)?;

    let visible_len = visible.len();
    let visible: Vec<json::EntrySearchResult> = visible
        .into_iter()
        .take(visible_len.min(search_limit))
        .map(Into::into)
        .collect();

    let invisible_len = invisible.len();
    let invisible: Vec<json::EntrySearchResult> = invisible
        .into_iter()
        .take(invisible_len.min(search_limit - search_limit.min(visible.len())))
        .map(Into::into)
        .collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}
