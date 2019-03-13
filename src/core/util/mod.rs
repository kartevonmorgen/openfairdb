pub mod filter;
pub mod geo;
pub mod nonce;
pub mod parse;
pub mod password;
pub mod rowid;
pub mod sort;
pub mod time;
pub mod validate;

use regex::Regex;

pub const ID_LIST_SEPARATOR: char = ',';

pub fn split_ids(ids: &str) -> Vec<&str> {
    ids.split(ID_LIST_SEPARATOR)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .collect()
}

lazy_static! {
    static ref HASH_TAG_REGEX: Regex = Regex::new(r"#(?P<tag>\w+((-\w+)*)?)").unwrap();
}

pub fn extract_hash_tags(text: &str) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for cap in HASH_TAG_REGEX.captures_iter(text) {
        res.push(cap["tag"].into());
    }
    res
}

pub fn remove_hash_tags(text: &str) -> String {
    HASH_TAG_REGEX
        .replace_all(text, "")
        .into_owned()
        .replace("  ", " ")
        .replace(",", "")
        .trim()
        .into()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn split_ids_test() {
        assert_eq!(split_ids("abc"), vec!["abc"]);
        assert_eq!(split_ids("a, b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_ids("\t").len(), 0);
        assert_eq!(split_ids("abc, ,d,"), vec!["abc", "d"]);
    }

    #[test]
    fn extract_single_hash_tag_from_text() {
        assert_eq!(extract_hash_tags("none").len(), 0);
        assert_eq!(extract_hash_tags("#").len(), 0);
        assert_eq!(extract_hash_tags("foo #bar none"), vec!["bar".to_string()]);
        assert_eq!(extract_hash_tags("foo #bar,none"), vec!["bar".to_string()]);
        assert_eq!(extract_hash_tags("foo#bar,none"), vec!["bar".to_string()]);
        assert_eq!(
            extract_hash_tags("foo#bar none#baz"),
            vec!["bar".to_string(), "baz".to_string()]
        );
        assert_eq!(
            extract_hash_tags("#bar#baz"),
            vec!["bar".to_string(), "baz".to_string()]
        );
        assert_eq!(
            extract_hash_tags("#a-long-tag#baz"),
            vec!["a-long-tag".to_string(), "baz".to_string()]
        );
        assert_eq!(extract_hash_tags("#-").len(), 0);
        assert_eq!(extract_hash_tags("#tag-"), vec!["tag".to_string()]);
    }

    #[test]
    fn remove_hash_tag_from_text() {
        assert_eq!(remove_hash_tags("some #tag"), "some");
        assert_eq!(remove_hash_tags("some#tag"), "some");
        assert_eq!(remove_hash_tags("#tag"), "");
        assert_eq!(remove_hash_tags("some #text with #tags"), "some with");
    }

}
