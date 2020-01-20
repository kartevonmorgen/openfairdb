use url::Url;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Links {
    pub homepage: Option<Url>,
    pub image: Option<Url>,
    pub image_href: Option<Url>,
}
