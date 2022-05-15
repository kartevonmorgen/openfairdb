use ofdb_entities::url::{ParseError, Url};

use crate::core::error::ParameterError;

/// Completes incomplete URLs before parsing
pub fn parse_lazy_url(url: &str) -> Result<Option<Url>, ParseError> {
    let url = url.trim();
    if url.is_empty() {
        return Ok(None);
    }
    if url.contains("://") {
        Url::parse(url).map(Some)
    } else {
        // Add the missing protocol by assuming https
        if url.starts_with("www.") {
            Url::parse(&format!("https://{}", url)).map(Some)
        } else {
            Url::parse(&format!("https://www.{}", url)).map(Some)
        }
    }
}

pub fn parse_url_param(url: &str) -> Result<Option<Url>, ParameterError> {
    parse_lazy_url(url).map_err(|_| ParameterError::Url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_params() {
        assert_eq!(None, parse_url_param("").unwrap());
        assert_eq!(None, parse_url_param("\t \n").unwrap());
        assert_eq!(
            parse_url_param("example.com/index.html").unwrap().unwrap(),
            "https://www.example.com/index.html".parse().unwrap()
        );
        assert_eq!(
            parse_url_param("www.example.com").unwrap().unwrap(),
            "https://www.example.com/".parse().unwrap()
        );
        assert_eq!(
            parse_url_param("http://www.example.com/index.html")
                .unwrap()
                .unwrap(),
            "http://www.example.com/index.html".parse().unwrap()
        );
        assert_eq!(
            parse_url_param("https://example.com").unwrap().unwrap(),
            "https://example.com/".parse().unwrap()
        );
    }
}
