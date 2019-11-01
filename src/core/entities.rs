use crate::core::util::{
    geo::{MapBbox, MapPoint},
    nonce::Nonce,
    password::Password,
    time::Timestamp,
};

use chrono::prelude::*;
use failure::{bail, format_err, Fallible};
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// Universal, external/public identifier with a string representation.
#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Uid(String);

impl Uid {
    pub fn new_uuid() -> Self {
        Uuid::new_v4().into()
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_empty()
    }
}

impl AsRef<str> for Uid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for Uid {
    fn from(from: String) -> Self {
        Uid(from)
    }
}

impl From<&str> for Uid {
    fn from(from: &str) -> Self {
        from.to_string().into()
    }
}

impl From<Uuid> for Uid {
    fn from(from: Uuid) -> Self {
        Self(from.to_simple_ref().to_string())
    }
}

impl From<Uid> for String {
    fn from(from: Uid) -> Self {
        from.0
    }
}

impl FromStr for Uid {
    type Err = ();
    fn from_str(s: &str) -> Result<Uid, Self::Err> {
        Ok(Uid(s.to_owned()))
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(&self.0)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Status(i16);

impl Status {
    pub const fn rejected() -> Self {
        Self(-1)
    }

    pub const fn archived() -> Self {
        Self(0)
    }

    pub const fn created() -> Self {
        Self(1)
    }

    pub const fn confirmed() -> Self {
        Self(2)
    }

    pub fn is_valid(self) -> bool {
        self.0 >= -1 && self.0 <= 2
    }

    pub const fn default() -> Self {
        Self::created()
    }

    pub const fn from_inner(inner: i16) -> Self {
        Self(inner)
    }

    pub const fn into_inner(self) -> i16 {
        self.0
    }

}

impl From<Status> for i16 {
    fn from(from: Status) -> Self {
        from.0
    }
}

impl From<i16> for Status {
    fn from(from: i16) -> Self {
        let status = Status::from_inner(from);
        debug_assert!(status.is_valid());
        status
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub uid            : Uid,
    pub version        : u64,
    pub created_at     : Timestamp,
    pub archived_at    : Option<Timestamp>,
    pub title          : String,
    pub description    : String,
    pub location       : Location,
    pub contact        : Option<Contact>,
    pub homepage       : Option<String>,
    pub categories     : Vec<Uid>,
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
    pub email : Option<String>,
    pub phone : Option<String>,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        !(self.email.is_some() || self.phone.is_some())
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub uid          : Uid,
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
    pub image_url     : Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegistrationType {
    Email,
    Phone,
    Homepage,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Category {
    pub uid : Uid,
    pub tag : String
}

impl Category {
    pub fn name(&self) -> String {
        format!("#{}", self.tag)
    }
}

impl Category {
    pub const UID_NON_PROFIT: &'static str = "2cd00bebec0c48ba9db761da48678134";
    pub const UID_COMMERCIAL: &'static str = "77b3c33a92554bcf8e8c2c86cedd6f6f";
    pub const UID_EVENT: &'static str = "c2dc278a2d6a4b9b8a50cb606fc017ed";

    pub const TAG_NON_PROFIT: &'static str = "non-profit";
    pub const TAG_COMMERCIAL: &'static str = "commercial";
    pub const TAG_EVENT: &'static str = "event";

    pub fn new_non_profit() -> Self {
        Self {
            uid: Self::UID_NON_PROFIT.into(),
            tag: Self::TAG_NON_PROFIT.into(),
        }
    }

    pub fn new_commercial() -> Self {
        Self {
            uid: Self::UID_COMMERCIAL.into(),
            tag: Self::TAG_COMMERCIAL.into(),
        }
    }

    pub fn new_event() -> Self {
        Self {
            uid: Self::UID_EVENT.into(),
            tag: Self::TAG_EVENT.into(),
        }
    }

    pub fn split_from_tags(tags: Vec<String>) -> (Vec<String>, Vec<Category>) {
        let mut categories = Vec::with_capacity(3);
        let tags = tags
            .into_iter()
            .filter(|t| match t.as_str() {
                Self::TAG_NON_PROFIT => {
                    categories.push(Self::new_non_profit());
                    false
                }
                Self::TAG_COMMERCIAL => {
                    categories.push(Self::new_commercial());
                    false
                }
                Self::TAG_EVENT => {
                    categories.push(Self::new_event());
                    false
                }
                _ => true,
            })
            .collect();
        (tags, categories)
    }

    pub fn merge_uids_into_tags(uids: Vec<Uid>, mut tags: Vec<String>) -> Vec<String> {
        tags.reserve(uids.len());
        tags = uids
            .iter()
            .fold(tags, |mut tags, uid| {
                match uid.as_ref() {
                    Self::UID_NON_PROFIT => tags.push(Self::TAG_NON_PROFIT.into()),
                    Self::UID_COMMERCIAL => tags.push(Self::TAG_COMMERCIAL.into()),
                    Self::UID_EVENT => tags.push(Self::TAG_EVENT.into()),
                    _ => (),
                }
                tags
            });
        tags.sort_unstable();
        tags.dedup();
        tags
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Tag {
    pub id: String,
}

pub type TagCount = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagFrequency(pub String, pub TagCount);

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub email           : String,
    pub email_confirmed : bool,
    pub password        : Password,
    pub role            : Role,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
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
    pub uid         : Uid,
    pub rating_uid  : Uid,
    pub created_at  : Timestamp,
    pub archived_at : Option<Timestamp>,
    pub text        : String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

impl RatingContext {
    // The number of different contexts, i.e. the number of enum variants
    pub const fn total_count() -> u8 {
        6
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, Eq, PartialEq, PartialOrd, Ord)]
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
        f64::from(from.0)
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
            / f64::from(RatingContext::total_count()))
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
    pub uid         : Uid,
    pub place_uid   : Uid,
    pub created_at  : Timestamp,
    pub archived_at : Option<Timestamp>,
    pub title       : String,
    pub value       : RatingValue,
    pub context     : RatingContext,
    pub source      : Option<String>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub struct BboxSubscription {
    pub uid        : Uid,
    pub user_email : String,
    pub bbox       : MapBbox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub owned_tags: Vec<String>,
    pub api_token: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UserToken {
    pub email_nonce: EmailNonce,
    pub expires_at: Timestamp,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EmailNonce {
    pub email: String,
    pub nonce: Nonce,
}

impl EmailNonce {
    pub fn encode_to_string(&self) -> String {
        let nonce = self.nonce.to_string();
        debug_assert_eq!(Nonce::STR_LEN, nonce.len());
        let mut concat = String::with_capacity(self.email.len() + nonce.len());
        concat += &self.email;
        concat += &nonce;
        bs58::encode(concat).into_string()
    }

    pub fn decode_from_str(encoded: &str) -> Fallible<EmailNonce> {
        let decoded = bs58::decode(encoded).into_vec()?;
        let mut concat = String::from_utf8(decoded)?;
        if concat.len() < Nonce::STR_LEN {
            bail!(
                "Invalid token - too short: {} <= {}",
                concat.len(),
                Nonce::STR_LEN
            );
        }
        let email_len = concat.len() - Nonce::STR_LEN;
        let nonce_slice: &str = &concat[email_len..];
        let nonce = nonce_slice
            .parse::<Nonce>()
            .map_err(|err| format_err!("Failed to parse nonce from '{}': {}", nonce_slice, err))?;
        concat.truncate(email_len);
        let email = concat;
        Ok(Self { email, nonce })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_email_nonce() {
        let example = EmailNonce {
            email: "test@example.com".into(),
            nonce: Nonce::new(),
        };
        let encoded = example.encode_to_string();
        let decoded = EmailNonce::decode_from_str(&encoded).unwrap();
        assert_eq!(example, decoded);
    }

    #[test]
    fn encode_decode_email_nonce_with_empty_email() {
        let example = EmailNonce {
            email: "".into(),
            nonce: Nonce::new(),
        };
        let encoded = example.encode_to_string();
        let decoded = EmailNonce::decode_from_str(&encoded).unwrap();
        assert_eq!(example, decoded);
    }

    #[test]
    fn decode_empty_email_nonce() {
        assert!(EmailNonce::decode_from_str("").is_err());
    }
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

    pub struct EntryBuild {
        entry: Entry,
    }

    impl EntryBuild {
        pub fn id(mut self, uid: &str) -> Self {
            self.entry.uid = uid.into();
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
                    uid: Uid::new_uuid(),
                    version: 0,
                    created_at: 0.into(),
                    archived_at: None,
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
        c.phone = Some("123".into());
        assert_eq!(c.is_empty(), false);
    }
}
