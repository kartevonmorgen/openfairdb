use crate::core::{prelude::*, util};
use ofdb_core::{bbox, tag};
use ofdb_entities::geo::MapBbox;

use std::collections::HashMap;

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a> {
    pub bbox       : MapBbox,
    pub ids        : Vec<&'a str>,
    pub categories : Vec<&'a str>,
    pub org_tag   :  Option<&'a str>,
    pub hash_tags  : Vec<&'a str>,
    pub text       : Option<&'a str>,
    pub status     : Vec<ReviewStatus>,
}

pub fn clear_search_results<D: Db>(
    db: &D,
    org_id: &Id,
    org_tag: &str,
    results: Vec<IndexedPlace>,
) -> Result<Vec<IndexedPlace>> {
    let place_ids: Vec<_> = results.iter().map(|p| p.id.as_str()).collect();
    let pending_clearances = db.load_pending_clearances_for_places(org_id, &place_ids)?;
    if pending_clearances.is_empty() {
        // No filtering required
        return Ok(results);
    }
    let pending_clearances: HashMap<_, _> = pending_clearances
        .into_iter()
        .map(|p| (p.place_id.to_string(), p))
        .collect();
    let mut cleared_results = Vec::with_capacity(results.len());
    for mut place in results.into_iter() {
        debug_assert!(place
            .tags
            .iter()
            .map(String::as_str)
            .find(|tag| *tag == org_tag)
            .is_some());
        let pending_clearance = pending_clearances.get(&place.id);
        if let Some(pending_clearance) = pending_clearance {
            if let Some(last_cleared_revision) = &pending_clearance.last_cleared_revision {
                let (last_cleared_place, current_status) =
                    db.load_place_revision(&place.id, *last_cleared_revision)?;
                debug_assert_eq!(*last_cleared_revision, last_cleared_place.revision);
                let Place {
                    description,
                    id,
                    location: Location { pos, .. },
                    tags,
                    title,
                    ..
                } = last_cleared_place;
                if tags
                    .iter()
                    .map(String::as_str)
                    .find(|tag| *tag == org_tag)
                    .is_none()
                {
                    // Remove previously untagged places from the result
                    continue;
                }
                // Ratings are independent of the revision
                let ratings = place.ratings;
                // Replace the actual/current search result item with the last cleared revision
                place = IndexedPlace {
                    id: id.into(),
                    description,
                    pos,
                    ratings,
                    status: Some(current_status),
                    tags,
                    title,
                };
            } else {
                // Skip newly created but not yet cleared entry
                continue;
            }
        }
        cleared_results.push(place);
    }
    Ok(cleared_results)
}

pub fn search<D: Db>(
    db: &D,
    index: &dyn PlaceIndex,
    req: SearchRequest,
    limit: usize,
) -> Result<(Vec<IndexedPlace>, Vec<IndexedPlace>)> {
    let SearchRequest {
        bbox: visible_bbox,
        ids,
        categories,
        org_tag,
        hash_tags: req_hash_tags,
        text,
        status,
    } = req;

    let mut hash_tags = text.map(util::extract_hash_tags).unwrap_or_default();
    hash_tags.reserve(req_hash_tags.len() + 1);
    for hash_tag in req_hash_tags {
        hash_tags.push(hash_tag.to_owned());
    }
    if let Some(org_tag) = org_tag {
        hash_tags.push(org_tag.to_owned());
    }

    let text = text.map(util::remove_hash_tags).and_then(|text| {
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

    let visible_places_query = IndexQuery {
        include_bbox: Some(visible_bbox),
        exclude_bbox: None,
        categories,
        ids,
        hash_tags,
        text_tags,
        text,
        status: Some(status),
        ..Default::default()
    };

    // 1st query: Search for visible results only
    // This is required to reliably retrieve all available results!
    // See also: https://github.com/slowtec/openfairdb/issues/183
    let mut visible_places = index
        .query_places(&visible_places_query, limit)
        .map_err(RepoError::Other)?;
    debug_assert!(visible_places
        .iter()
        .all(|e| visible_bbox.contains_point(e.pos)));
    if let Some(org_tag) = org_tag {
        if let Some(org_id) = db.map_tag_to_clearance_org_id(org_tag)? {
            visible_places = clear_search_results(db, &org_id, org_tag, visible_places)?;
        }
    }

    // 2nd query: Search for remaining invisible results
    let mut invisible_places = if visible_places.len() < limit {
        let invisible_places_query = IndexQuery {
            include_bbox: Some(bbox::extend_bbox(&visible_bbox)),
            exclude_bbox: visible_places_query.include_bbox,
            ..visible_places_query
        };
        index
            .query_places(&invisible_places_query, limit - visible_places.len())
            .map_err(RepoError::Other)?
    } else {
        vec![]
    };
    debug_assert!(!invisible_places
        .iter()
        .any(|e| visible_bbox.contains_point(e.pos)));
    if let Some(org_tag) = org_tag {
        if let Some(org_id) = db.map_tag_to_clearance_org_id(org_tag)? {
            invisible_places = clear_search_results(db, &org_id, &org_tag, invisible_places)?;
        }
    }

    Ok((visible_places, invisible_places))
}

/// The global search usecase is like the one
/// of usual internet search engines that exists
/// of only one single search input.
/// So here we don't care about tags, categories etc.
/// We also ignore the rating of an entry for now.
pub fn global_search(index: &dyn PlaceIndex, txt: &str, limit: usize) -> Result<Vec<IndexedPlace>> {
    let index_query = IndexQuery {
        text: Some(txt.into()),
        ..Default::default()
    };

    let entries = index
        .query_places(&index_query, limit)
        .map_err(RepoError::Other)?;

    Ok(entries)
}
