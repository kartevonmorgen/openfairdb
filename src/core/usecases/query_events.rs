use super::EventQuery;
use crate::core::{
    prelude::*,
    util::{extract_hash_tags, remove_hash_tags},
};
use ofdb_core::{bbox, tag};

const DEFAULT_RESULT_LIMIT: usize = 100;

#[allow(clippy::absurd_extreme_comparisons)]
pub fn query_events<D: Db>(db: &D, index: &dyn IdIndex, query: EventQuery) -> Result<Vec<Event>> {
    if query.is_empty() {
        // Special case for backwards compatibility
        return Ok(db.all_events_chronologically()?);
    }
    let EventQuery {
        bbox: visible_bbox,
        created_by,
        start_min,
        start_max,
        end_min,
        end_max,
        tags,
        text,
        limit,
    } = query;

    let mut hash_tags = text.as_deref().map(extract_hash_tags).unwrap_or_default();
    if let Some(tags) = tags {
        hash_tags.reserve(hash_tags.len() + tags.len());
        for hashtag in tags {
            hash_tags.push(hashtag.to_owned());
        }
    }

    let text = text.as_deref().map(remove_hash_tags).and_then(|text| {
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    });

    let text_tags = text
        .as_deref()
        .map(tag::split_text_into_tags)
        .unwrap_or_default();

    let visible_events_query = IndexQuery {
        include_bbox: visible_bbox,
        exclude_bbox: None,
        categories: vec![Category::ID_EVENT],
        hash_tags,
        text_tags,
        text,
        ts_min_lb: start_min,
        ts_min_ub: start_max,
        ts_max_lb: end_min,
        ts_max_ub: end_max,
        ..Default::default()
    };

    let limit = limit.unwrap_or_else(|| {
        info!(
            "No limit requested - Using default limit {} for event search results",
            DEFAULT_RESULT_LIMIT
        );
        DEFAULT_RESULT_LIMIT
    });

    // 1st query: Search for visible results only
    // This is required to reliably retrieve all available results!
    // See also: https://github.com/slowtec/openfairdb/issues/183
    let visible_event_ids = index
        .query_ids(IndexQueryMode::WithoutRating, &visible_events_query, limit)
        .map_err(RepoError::Other)?;

    // 2nd query: Search for remaining invisible results
    let invisible_event_ids = if let Some(visible_bbox) = visible_bbox {
        if visible_event_ids.len() < limit {
            let invisible_events_query = IndexQuery {
                include_bbox: Some(bbox::extend_bbox(&visible_bbox)),
                exclude_bbox: visible_events_query.include_bbox,
                ..visible_events_query
            };
            index
                .query_ids(
                    IndexQueryMode::WithoutRating,
                    &invisible_events_query,
                    limit - visible_event_ids.len(),
                )
                .map_err(RepoError::Other)?
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let event_ids: Vec<_> = visible_event_ids
        .iter()
        .chain(invisible_event_ids.iter())
        .map(Id::as_str)
        .collect();
    let mut events = db.get_events_chronologically(&event_ids)?;

    if let Some(ref email) = created_by {
        if let Some(user) = db.try_get_user_by_email(email)? {
            events = events
                .into_iter()
                .filter(|e| e.created_by.as_ref() == Some(&user.email))
                .collect();
        } else {
            events = vec![];
        }
    }

    Ok(events)
}
