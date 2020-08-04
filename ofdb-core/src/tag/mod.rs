use crate::text;

pub mod moderated;

pub fn split_text_into_tags(text: &str) -> Vec<String> {
    text::split_text_into_words(text)
        .into_iter()
        .map(str::to_lowercase)
        .collect()
}
