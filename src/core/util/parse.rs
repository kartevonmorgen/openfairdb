use crate::core::error::ParameterError;
use url::{ParseError, Url};

/// Completes incomplete URLs before parsing
pub fn parse_lazy_url(url: &str) -> Result<Url, ParseError> {
    let url = url.trim();
    if url.is_empty() || url.contains("://") {
        Url::parse(url)
    } else {
        // Add the missing protocol by assuming https
        if url.starts_with("www.") {
            Url::parse(&format!("https://{}", url))
        } else {
            Url::parse(&format!("https://www.{}", url))
        }
    }
}

pub fn parse_url_param(url: &str) -> Result<String, ParameterError> {
    parse_lazy_url(url)
        .map(|url| url.into_string())
        .map_err(|_| ParameterError::Url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_param() {
        assert!(super::parse_url_param("").is_err());
        assert!(super::parse_url_param("\t \n").is_err());
        assert_eq!(
            super::parse_url_param("example.com/index.html").unwrap(),
            "https://www.example.com/index.html"
        );
        assert_eq!(
            super::parse_url_param("www.example.com").unwrap(),
            "https://www.example.com/"
        );
        assert_eq!(
            super::parse_url_param("http://www.example.com/index.html").unwrap(),
            "http://www.example.com/index.html"
        );
        assert_eq!(
            super::parse_url_param("https://example.com").unwrap(),
            "https://example.com/"
        );
    }
}
