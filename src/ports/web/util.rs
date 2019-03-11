use regex::Regex;

lazy_static! {
    static ref HASH_TAG_REGEX: Regex = Regex::new(r"#(?P<tag>\w+((-\w+)*)?)").unwrap();
}

pub const ID_LIST_SEPARATOR: char = ',';

pub fn split_ids(ids: &str) -> Vec<&str> {
    ids.split(ID_LIST_SEPARATOR)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .collect()
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
