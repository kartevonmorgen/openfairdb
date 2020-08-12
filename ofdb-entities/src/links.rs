use url::Url;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Links {
    pub homepage: Option<Url>,
    pub image: Option<Url>,
    pub image_href: Option<Url>,
    pub custom: Vec<CustomLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomLink {
    pub url: Url,
    pub title: Option<String>,
    pub description: Option<String>,
}

impl CustomLink {
    pub const fn from_url(url: Url) -> Self {
        Self {
            url,
            title: None,
            description: None,
        }
    }
}

impl From<Url> for CustomLink {
    fn from(url: Url) -> Self {
        Self::from_url(url)
    }
}
