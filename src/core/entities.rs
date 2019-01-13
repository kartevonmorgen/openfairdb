#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id             : String,
    pub osm_node       : Option<u64>,
    pub created        : u64,
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

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Location {
    //TODO: use Coordinate
    pub lat     : f64,
    pub lng     : f64,
    pub address : Option<Address>
}

#[cfg_attr(rustfmt, rustfmt_skip)]
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

#[cfg_attr(rustfmt, rustfmt_skip)]
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

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id           : String,
    pub title        : String,
    pub description  : Option<String>,
    pub start        : u64,
    pub end          : Option<u64>,
    pub location     : Option<Location>,
    pub contact      : Option<Contact>,
    pub tags         : Vec<String>,
    pub homepage     : Option<String>,
    pub created_by   : Option<String>,
    pub registration : Option<RegistrationType>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegistrationType {
    Email,
    Phone,
    Homepage,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Category {
    pub id      : String,
    pub created : u64,
    pub version : u64,
    pub name    : String
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    pub id: String,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id              : String, // TODO: remove
    pub username        : String,
    pub password        : String,
    pub email           : String,
    pub email_confirmed : bool,
    pub role            : Role,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
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

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub id        : String,
    pub created   : u64,
    pub text      : String,
    pub rating_id : String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RatingContext {
    Diversity,
    Renewable,
    Fairness,
    Humanity,
    Transparency,
    Solidarity,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct Rating {
    pub id       : String,
    pub entry_id : String,
    pub created  : u64,
    pub title    : String,
    pub value    : i8,
    pub context  : RatingContext,
    pub source   : Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bbox {
    pub south_west: Coordinate,
    pub north_east: Coordinate,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Debug, Clone, PartialEq)]
pub struct BboxSubscription {
    pub id       : String,
    pub bbox     : Bbox,
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
        pub fn lat(mut self, lat: f64) -> Self {
            self.entry.location.lat = lat;
            self
        }
        pub fn lng(mut self, lng: f64) -> Self {
            self.entry.location.lng = lng;
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
                    created: 0,
                    version: 0,
                    title: "".into(),
                    description: "".into(),
                    location: Location {
                        lat: 0.0,
                        lng: 0.0,
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
