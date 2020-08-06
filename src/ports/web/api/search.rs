use crate::{
    adapters::json,
    core::{
        prelude::*,
        usecases,
        util::{self, geo},
    },
    infrastructure::{
        db::{sqlite, tantivy},
        error::AppError,
    },
};

use rocket::{self, request::Form};
use rocket_contrib::json::Json;
use std::result;

#[derive(FromForm, Clone)]
pub struct SearchQuery {
    bbox: String,
    categories: Option<String>,
    ids: Option<String>,
    org_tag: Option<String>,
    tags: Option<String>,
    text: Option<String>,
    status: Option<String>,
    limit: Option<usize>,
}

pub fn parse_search_query(
    query: &'_ SearchQuery,
) -> result::Result<(usecases::SearchRequest<'_>, Option<usize>), AppError> {
    let SearchQuery {
        bbox,
        ids,
        categories,
        org_tag,
        tags,
        text,
        status,
        limit,
    } = query;

    let bbox = bbox
        .parse::<geo::MapBbox>()
        .map_err(|_| ParameterError::Bbox)
        .map_err(Error::Parameter)
        .map_err(AppError::Business)?;

    let ids = ids.as_deref().map(util::split_ids).unwrap_or_default();

    let categories = categories
        .as_deref()
        .map(util::split_ids)
        .map(|ids| {
            ids.into_iter()
                // Only places, not events
                .filter(|id| id != &Category::ID_EVENT)
                .collect()
        })
        .unwrap_or_default();

    let hash_tags = tags.as_deref().map(util::split_ids).unwrap_or_default();

    let text = text.as_deref();

    let status = status
        .as_deref()
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

    Ok((
        usecases::SearchRequest {
            bbox,
            ids,
            categories,
            org_tag: org_tag.as_ref().map(String::as_str),
            hash_tags,
            text,
            status,
        },
        *limit,
    ))
}

type Result<T> = result::Result<Json<T>, AppError>;

const DEFAULT_RESULT_LIMIT: usize = 100;
const MAX_RESULT_LIMIT: usize = 500;

#[get("/search?<query..>")]
#[allow(clippy::absurd_extreme_comparisons)]
pub fn get_search(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    query: Form<SearchQuery>,
) -> Result<json::SearchResponse> {
    let query = query.into_inner();
    let (req, limit) = parse_search_query(&query)?;

    let limit = if let Some(limit) = limit {
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

    let (visible, invisible) =
        usecases::search(&*connections.shared()?, &search_engine, req, limit)?;

    let visible: Vec<json::PlaceSearchResult> = visible.into_iter().map(Into::into).collect();

    let invisible: Vec<json::PlaceSearchResult> = invisible.into_iter().map(Into::into).collect();

    Ok(Json(json::SearchResponse { visible, invisible }))
}
