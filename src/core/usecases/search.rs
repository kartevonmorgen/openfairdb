use crate::core::prelude::*;
use crate::core::util::{self, filter, geo::MapBbox};

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub struct SearchRequest<'a, 'b, 'c, 'd> {
    pub bbox       : MapBbox,
    pub ids        : Vec<&'b str>,
    pub categories : Vec<&'a str>,
    pub hash_tags  : Vec<&'c str>,
    pub text       : Option<&'d str>,
}

pub fn search(
    index: &EntryIndex,
    req: SearchRequest,
    limit: usize,
) -> Result<(Vec<IndexedEntry>, Vec<IndexedEntry>)> {
    let visible_bbox: MapBbox = req.bbox;

    let mut hash_tags = req.text.map(util::extract_hash_tags).unwrap_or_default();
    hash_tags.reserve(hash_tags.len() + req.hash_tags.len());
    for hashtag in req.hash_tags {
        hash_tags.push(hashtag.to_owned());
    }

    let text = req.text.map(util::remove_hash_tags).and_then(|text| {
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    });

    let text_tags = text
        .as_ref()
        .map(String::as_str)
        .map(filter::split_text_to_words)
        .unwrap_or_default();

    let index_query = EntryIndexQuery {
        bbox: Some(filter::extend_bbox(&visible_bbox)),
        categories: req.categories,
        ids: req.ids,
        hash_tags,
        text_tags,
        text,
    };

    let entries = index
        .query_entries(&index_query, limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;

    let (visible_entries, invisible_entries): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|e| visible_bbox.contains_point(e.pos));

    Ok((visible_entries, invisible_entries))
}

/// The global search usecase is like the one
/// of usual internet search engines that exists
/// of only one single search input.
/// So here we don't care about tags, categories etc.
/// We also ignore the rating of an entry for now.
pub fn global_search(index: &EntryIndex, txt: &str, limit: usize) -> Result<Vec<IndexedEntry>> {
    let index_query = EntryIndexQuery {
        text: Some(txt.into()),
        ..Default::default()
    };

    let entries = index
        .query_entries(&index_query, limit)
        .map_err(|err| RepoError::Other(Box::new(err.compat())))?;

    Ok(entries)
}
