use crate::core::{prelude::*, util};
use ofdb_core::util::filter;
use ofdb_entities::geo::MapBbox;

use std::collections::HashMap;

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a> {
    pub bbox       : MapBbox,
    pub ids        : Vec<&'a str>,
    pub categories : Vec<&'a str>,
    pub auth_tag   : Option<&'a str>,
    pub hash_tags  : Vec<&'a str>,
    pub text       : Option<&'a str>,
    pub status     : Vec<ReviewStatus>,
}

pub fn authorize_search_results<D: Db>(
    db: &D,
    org_id: &Id,
    results: Vec<IndexedPlace>,
) -> Result<Vec<IndexedPlace>> {
    let place_ids: Vec<_> = results.iter().map(|p| p.id.as_str()).collect();
    let pending_authorizations = db.load_pending_authorizations_for_places(org_id, &place_ids)?;
    if pending_authorizations.is_empty() {
        // No filtering required
        return Ok(results);
    }
    let pending_authorizations: HashMap<_, _> = pending_authorizations
        .into_iter()
        .map(|p| (p.place_id.to_string(), p))
        .collect();
    let mut authorized_results = Vec::with_capacity(results.len());
    for place in results.into_iter() {
        let pending_authorization = pending_authorizations.get(&place.id);
        if let Some(pending_authorization) = pending_authorization {
            if let Some(last_authorized) = &pending_authorization.last_authorized {
                if let Some(authorized_review_status) = last_authorized.review_status {
                    if !authorized_review_status.exists() {
                        // Skip previously archived/rejected entry that has later been restored
                        continue;
                    }
                }
                let last_authorized_place =
                    db.load_place_revision(&place.id, last_authorized.revision)?;
                log::warn!(
                    "TODO: Replace unauthorized {:?} with {:?} in search results instead of excluding it entirely",
                    place,
                    last_authorized_place
                );
                // Skip unauthorized entry until conversion from last_authorized_place into place is available
                // TODO: Update the OpenAPI docs that also mention this temporary workaround!
                continue; // TODO: Remove this line to include the entry in the authorized results
            } else {
                // Skip newly created but not yet authorized entry
                continue;
            }
        }
        authorized_results.push(place);
    }
    Ok(authorized_results)
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
        auth_tag,
        hash_tags: req_hash_tags,
        text,
        status,
    } = req;

    let mut hash_tags = text.map(util::extract_hash_tags).unwrap_or_default();
    hash_tags.reserve(req_hash_tags.len() + 1);
    for hash_tag in req_hash_tags {
        hash_tags.push(hash_tag.to_owned());
    }
    if let Some(auth_tag) = auth_tag {
        hash_tags.push(auth_tag.to_owned());
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
        .map(filter::split_text_to_words)
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
    if let Some(auth_tag) = auth_tag {
        if let Some(org_id) = db.map_authorized_tag_to_org_id(auth_tag)? {
            visible_places = authorize_search_results(db, &org_id, visible_places)?;
        }
    }

    // 2nd query: Search for remaining invisible results
    let mut invisible_places = if visible_places.len() < limit {
        let invisible_places_query = IndexQuery {
            include_bbox: Some(filter::extend_bbox(&visible_bbox)),
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
    if let Some(auth_tag) = auth_tag {
        if let Some(org_id) = db.map_authorized_tag_to_org_id(auth_tag)? {
            invisible_places = authorize_search_results(db, &org_id, invisible_places)?;
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
