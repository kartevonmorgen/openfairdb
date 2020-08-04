fn is_word_separator(c: char) -> bool {
    c.is_ascii_whitespace() || c == ',' || c == '.' || c == ';'
}

pub fn split_text_into_words(text: &str) -> Vec<&str> {
    text.split(is_word_separator)
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_split_text_into_words() {
        assert_eq!(
            vec!["A-a", "B", "#", "b", "C_c", "d", "-", "D"],
            split_text_into_words(" . A-a,B # b C_c;d - D , ")
        );
    }
}
