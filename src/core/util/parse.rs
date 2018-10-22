use core::error::ParameterError;
use url::{ParseError, Url};

/// Completes incomplete URLs before parsing
pub fn parse_lazy_url<S>(from: S) -> Result<Url, ParseError>
where
    S: Into<String>,
{
    let from = from.into();
    let from = from.trim();
    if from.is_empty() || from.contains("://") {
        Url::parse(from)
    } else {
        // Add the missing protocol by assuming https
        if from.starts_with("www.") {
            Url::parse(&format!("https://{}", from))
        } else {
            Url::parse(&format!("https://www.{}", from))
        }
    }
}

pub fn parse_url_param<S>(from: S) -> Result<String, ParameterError>
where
    S: Into<String>,
{
    parse_lazy_url(from)
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
