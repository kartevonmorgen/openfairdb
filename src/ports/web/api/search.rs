use crate::{
    adapters::json,
    core::{
        prelude::*,
        usecases,
        util::{self, geo},
    },
    infrastructure::{db::tantivy, error::AppError},
};

use rocket::{self, request::Form};
use rocket_contrib::json::Json;
use std::result;

#[derive(FromForm, Clone)]
pub struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    ids: Option<String>,
    tags: Option<String>,
    text: Option<String>,
    status: Option<String>,
    limit: Option<usize>,
}

type Result<T> = result::Result<Json<T>, AppError>;

const DEFAULT_RESULT_LIMIT: usize = 100;
const MAX_RESULT_LIMIT: usize = 500;

#[get("/search?<search..>")]
#[allow(clippy::absurd_extreme_comparisons)]
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

    let ids = search
        .ids
        .as_ref()
        .map(String::as_str)
        .map(util::split_ids)
        .unwrap_or_default();

    let categories = search
        .categories
        .as_ref()
        .map(String::as_str)
        .map(util::split_ids)
        .map(|ids| {
            ids.into_iter()
                // Only places, not events
                .filter(|id| id != &Category::ID_EVENT)
                .collect()
        })
        .unwrap_or_default();

    let hash_tags = search
        .tags
        .as_ref()
        .map(String::as_str)
        .map(util::split_ids)
        .unwrap_or_default();

    let text = search.text.as_ref().map(String::as_str);

    let status = search
        .status
        .as_ref()
        .map(String::as_str)
        .map(util::split_ids)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| {
            serde_json::from_str::<json::ReviewStatus>(&format!("\"{}\"", s))
                .map_err(|e| {
                    log::warn!("Failed to parse status '{}' from search query: {}", s, e);
                    e
                })
                .map(ReviewStatus::from)
                .ok()
        })
        .collect();

    let req = usecases::SearchRequest {
        bbox,
        ids,
        categories,
        hash_tags,
        text,
        status,
    };

    let search_limit = if let Some(limit) = search.limit {
        if limit > MAX_RESULT_LIMIT {
            info!(
                "Requested limit {} exceeds maximum limit {} for search results",
                limit, MAX_RESULT_LIMIT
            );
            MAX_RESULT_LIMIT
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
            "No limit requested - Using default limit {} for search results",
            DEFAULT_RESULT_LIMIT
        );
        DEFAULT_RESULT_LIMIT
    };

    let (visible, invisible) = usecases::search(&search_engine, req, search_limit)?;

    let visible: Vec<json::PlaceSearchResult> = visible.into_iter().map(Into::into).collect();

    let invisible: Vec<json::PlaceSearchResult> = invisible.into_iter().map(Into::into).collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}
