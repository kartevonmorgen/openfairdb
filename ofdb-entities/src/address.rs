#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Address {
    pub street  : Option<String>,
    pub zip     : Option<String>,
    pub city    : Option<String>,
    pub country : Option<String>,
    pub state   : Option<String>,
}

impl Address {
    pub fn is_empty(&self) -> bool {
        self.street.is_none()
            && self.zip.is_none()
            && self.city.is_none()
            && self.country.is_none()
            && self.state.is_none()
    }
}
