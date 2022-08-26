//! Search result item

use crate::boundary::PlaceSearchResult;
use seed::{prelude::*, *};

#[derive(Clone)]
pub struct Mdl {
    pub max_desc_len: usize,
    pub max_tag_count: usize,
    pub max_tag_len: usize,
    pub max_title_len: usize,
}

impl Default for Mdl {
    fn default() -> Self {
        Self {
            max_desc_len: 80,
            max_tag_count: 4,
            max_tag_len: 15,
            max_title_len: 40,
        }
    }
}

pub fn view<M>(mdl: &Mdl, p: &PlaceSearchResult) -> Node<M> {
    div![
        h3![trim(&p.title, mdl.max_title_len)],
        p![trim(&p.description, mdl.max_desc_len)],
        ul![p
            .tags
            .iter()
            .take(mdl.max_tag_count)
            .map(|t| span![trim(t, mdl.max_tag_len)])]
    ]
}

const ELLIPSIS: &str = "...";

fn trim(s: &str, max_len: usize) -> String {
    if s.len() > 3 && s.len() + 3 > max_len {
        s.chars()
            .take(max_len - 3)
            .chain(ELLIPSIS.chars())
            .collect::<String>()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_to_max_len() {
        assert_eq!(trim("This is very looong", 5), "Th...");
        assert_eq!(trim("One", 3), "One");
        assert_eq!(trim("This is short", 20), "This is short");
    }
}
