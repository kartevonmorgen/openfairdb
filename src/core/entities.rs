use chrono::prelude::*;

use crate::core::util::{
    geo::{MapBbox, MapPoint},
    password::Password,
    time::Timestamp,
};

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id             : String,
    pub osm_node       : Option<u64>,
    pub created        : Timestamp,
    pub archived       : Option<Timestamp>,
    pub version        : u64,
    pub title          : String,
    pub description    : String,
    pub location       : Location,
    pub contact        : Option<Contact>,
    pub homepage       : Option<String>,
    pub categories     : Vec<String>,
    pub tags           : Vec<String>,
    pub license        : Option<String>,
    pub image_url      : Option<String>,
    pub image_link_url : Option<String>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Location {
    pub pos:      MapPoint,
    pub address : Option<Address>
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq,Default)]
pub struct Address {
    pub street  : Option<String>,
    pub zip     : Option<String>,
    pub city    : Option<String>,
    pub country : Option<String>,
}

impl Address {
    pub fn is_empty(&self) -> bool {
        !(self.street.is_some()
            || self.zip.is_some()
            || self.city.is_some()
            || self.country.is_some())
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Contact {
    pub email     : Option<String>,
    pub telephone : Option<String>,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        !(self.email.is_some() || self.telephone.is_some())
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id           : String,
    pub title        : String,
    pub description  : Option<String>,
    pub start        : NaiveDateTime,
    pub end          : Option<NaiveDateTime>,
    pub location     : Option<Location>,
    pub contact      : Option<Contact>,
    pub tags         : Vec<String>,
    pub homepage     : Option<String>,
    pub created_by   : Option<String>,
    pub registration : Option<RegistrationType>,
    pub organizer    : Option<String>,
    pub archived     : Option<Timestamp>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegistrationType {
    Email,
    Phone,
    Homepage,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Category {
    pub id      : String,
    pub created : i64,
    pub version : u64,
    pub name    : String
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id              : String, // TODO: remove
    pub username        : String,
    pub password        : Password,
    pub email           : String,
    pub email_confirmed : bool,
    pub role            : Role,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Role {
    Guest = 0,
    User  = 1,
    Scout = 2,
    Admin = 3,
}

impl Default for Role {
    fn default() -> Role {
        Role::Guest
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub id        : String,
    pub created   : Timestamp,
    pub archived  : Option<Timestamp>,
    pub text      : String,
    pub rating_id : String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RatingValue(i8);

impl RatingValue {
    pub fn new<I: Into<i8>>(val: I) -> Self {
        let new = Self(val.into());
        debug_assert!(new.is_valid());
        new
    }

    pub const fn min() -> Self {
        Self(-1)
    }

    pub const fn max() -> Self {
        Self(2)
    }

    pub fn clamp(self) -> Self {
        Self(self.0.max(Self::min().0).min(Self::max().0))
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<i8> for RatingValue {
    fn from(from: i8) -> Self {
        Self(from)
    }
}

impl From<RatingValue> for i8 {
    fn from(from: RatingValue) -> Self {
        from.0
    }
}

impl From<RatingValue> for f64 {
    fn from(from: RatingValue) -> Self {
        from.0 as f64
    }
}

impl std::ops::Add for AvgRatingValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for AvgRatingValue {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl std::ops::Div<f64> for AvgRatingValue {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs)
    }
}

impl std::ops::DivAssign<f64> for AvgRatingValue {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct AvgRatingValue(f64);

impl AvgRatingValue {
    pub const fn min() -> Self {
        Self(-1.0)
    }

    pub const fn max() -> Self {
        Self(2.0)
    }

    pub fn clamp(self) -> Self {
        Self(self.0.max(Self::min().0).min(Self::max().0))
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<f64> for AvgRatingValue {
    fn from(from: f64) -> Self {
        Self(from)
    }
}

impl From<AvgRatingValue> for f64 {
    fn from(from: AvgRatingValue) -> Self {
        from.0
    }
}

impl From<RatingValue> for AvgRatingValue {
    fn from(from: RatingValue) -> Self {
        f64::from(i8::from(from)).into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct AvgRatingValueBuilder {
    acc: i64,
    cnt: usize,
}

impl AvgRatingValueBuilder {
    fn add(&mut self, val: RatingValue) {
        debug_assert!(val.is_valid());
        self.acc += i64::from(val.0);
        self.cnt += 1;
    }

    pub fn build(self) -> AvgRatingValue {
        if self.cnt > 0 {
            AvgRatingValue::from(self.acc as f64 / self.cnt as f64).clamp()
        } else {
            Default::default()
        }
    }
}

impl std::ops::AddAssign<RatingValue> for AvgRatingValueBuilder {
    fn add_assign(&mut self, rhs: RatingValue) {
        self.add(rhs);
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct AvgRatings {
    pub diversity: AvgRatingValue,
    pub fairness: AvgRatingValue,
    pub humanity: AvgRatingValue,
    pub renewable: AvgRatingValue,
    pub solidarity: AvgRatingValue,
    pub transparency: AvgRatingValue,
}

impl AvgRatings {
    pub fn total(&self) -> AvgRatingValue {
        ((self.diversity
            + self.fairness
            + self.humanity
            + self.renewable
            + self.solidarity
            + self.transparency)
            / 6.0)
            .clamp()
    }
}

#[derive(Debug, Default, Clone)]
pub struct AvgRatingsBuilder {
    pub diversity: AvgRatingValueBuilder,
    pub fairness: AvgRatingValueBuilder,
    pub humanity: AvgRatingValueBuilder,
    pub renewable: AvgRatingValueBuilder,
    pub solidarity: AvgRatingValueBuilder,
    pub transparency: AvgRatingValueBuilder,
}

impl AvgRatingsBuilder {
    pub fn add(&mut self, ctx: RatingContext, val: RatingValue) {
        use RatingContext::*;
        match ctx {
            Diversity => self.diversity.add(val),
            Fairness => self.fairness.add(val),
            Humanity => self.humanity.add(val),
            Renewable => self.renewable.add(val),
            Solidarity => self.solidarity.add(val),
            Transparency => self.transparency.add(val),
        }
    }

    pub fn build(self) -> AvgRatings {
        AvgRatings {
            diversity: self.diversity.build(),
            fairness: self.fairness.build(),
            humanity: self.humanity.build(),
            renewable: self.renewable.build(),
            solidarity: self.solidarity.build(),
            transparency: self.transparency.build(),
        }
    }
}

impl std::ops::AddAssign<(RatingContext, RatingValue)> for AvgRatingsBuilder {
    fn add_assign(&mut self, rhs: (RatingContext, RatingValue)) {
        self.add(rhs.0, rhs.1);
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Rating {
    pub id       : String,
    pub entry_id : String,
    pub created  : Timestamp,
    pub archived : Option<Timestamp>,
    pub title    : String,
    pub value    : RatingValue,
    pub context  : RatingContext,
    pub source   : Option<String>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct BboxSubscription {
    pub id       : String,
    pub bbox     : MapBbox,
    pub username : String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub owned_tags: Vec<String>,
    pub api_token: String,
}

#[cfg(test)]
pub trait Builder {
    type Build;
    fn build() -> Self::Build;
}

#[cfg(test)]
pub use self::entry_builder::*;

#[cfg(test)]
pub mod entry_builder {

    use super::*;
    use uuid::Uuid;

    pub struct EntryBuild {
        entry: Entry,
    }

    impl EntryBuild {
        pub fn id(mut self, id: &str) -> Self {
            self.entry.id = id.into();
            self
        }
        pub fn version(mut self, v: u64) -> Self {
            self.entry.version = v;
            self
        }
        pub fn title(mut self, title: &str) -> Self {
            self.entry.title = title.into();
            self
        }
        pub fn description(mut self, desc: &str) -> Self {
            self.entry.description = desc.into();
            self
        }
        pub fn pos(mut self, pos: MapPoint) -> Self {
            self.entry.location.pos = pos;
            self
        }
        pub fn categories(mut self, cats: Vec<&str>) -> Self {
            self.entry.categories = cats.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn tags(mut self, tags: Vec<&str>) -> Self {
            self.entry.tags = tags.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn license(mut self, license: Option<&str>) -> Self {
            self.entry.license = license.map(|s| s.into());
            self
        }
        pub fn image_url(mut self, image_url: Option<&str>) -> Self {
            self.entry.image_url = image_url.map(Into::into);
            self
        }
        pub fn image_link_url(mut self, image_link_url: Option<&str>) -> Self {
            self.entry.image_link_url = image_link_url.map(Into::into);
            self
        }
        pub fn finish(self) -> Entry {
            self.entry
        }
    }

    impl Builder for Entry {
        type Build = EntryBuild;
        fn build() -> EntryBuild {
            EntryBuild {
                entry: Entry {
                    id: Uuid::new_v4().to_simple_ref().to_string(),
                    osm_node: None,
                    created: 0.into(),
                    archived: None,
                    version: 0,
                    title: "".into(),
                    description: "".into(),
                    location: Location {
                        pos: MapPoint::from_lat_lng_deg(0.0, 0.0),
                        address: None,
                    },
                    contact: None,
                    homepage: None,
                    categories: vec![],
                    tags: vec![],
                    license: None,
                    image_url: None,
                    image_link_url: None,
                },
            }
        }
    }

}

#[cfg(test)]
pub use self::address_builder::*;

#[cfg(test)]
pub mod address_builder {

    use super::*;
    pub struct AddressBuild {
        addr: Address,
    }

    impl AddressBuild {
        pub fn street(mut self, x: &str) -> Self {
            self.addr.street = Some(x.into());
            self
        }
        pub fn zip(mut self, x: &str) -> Self {
            self.addr.zip = Some(x.into());
            self
        }
        pub fn city(mut self, x: &str) -> Self {
            self.addr.city = Some(x.into());
            self
        }
        pub fn country(mut self, x: &str) -> Self {
            self.addr.country = Some(x.into());
            self
        }
        pub fn finish(self) -> Address {
            self.addr
        }
    }

    impl Builder for Address {
        type Build = AddressBuild;
        fn build() -> Self::Build {
            AddressBuild {
                addr: Address::default(),
            }
        }
    }

    #[test]
    fn empty_address() {
        assert!(Address::default().is_empty());
        assert_eq!(Address::build().street("x").finish().is_empty(), false);
        assert_eq!(Address::build().zip("x").finish().is_empty(), false);
        assert_eq!(Address::build().city("x").finish().is_empty(), false);
        assert_eq!(Address::build().country("x").finish().is_empty(), false);
    }

    #[test]
    fn empty_contact() {
        assert!(Contact::default().is_empty());
        let mut c = Contact::default();
        c.email = Some("foo@bar".into());
        assert_eq!(c.is_empty(), false);
        let mut c = Contact::default();
        c.telephone = Some("123".into());
        assert_eq!(c.is_empty(), false);
    }
}
