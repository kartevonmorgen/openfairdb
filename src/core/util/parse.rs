use url::{ParseError, Url};

/// Completes incomplete URLs before parsing
pub fn parse_lazy_url<S>(from: S) -> Result<(String, Url), ParseError>
where
    S: Into<String>,
{
    let mut from = from.into();
    if !from.contains("://") {
        // Add the missing protocol by assuming https
        if from.starts_with("www.") {
            from = format!("https://{}", from);
        } else {
            from = format!("https://www.{}", from);
        }
    }
    let url = Url::parse(&from)?;
    Ok((from, url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lazy_url() {
        assert_eq!(
            super::parse_lazy_url("example.com/index.html").unwrap().0,
            "https://www.example.com/index.html"
        );
        assert_eq!(
            super::parse_lazy_url("www.example.com").unwrap().0,
            "https://www.example.com"
        );
        assert_eq!(
            super::parse_lazy_url("http://www.example.com/index.html")
                .unwrap()
                .0,
            "http://www.example.com/index.html"
        );
        assert_eq!(
            super::parse_lazy_url("https://example.com").unwrap().0,
            "https://example.com"
        );
    }
}
