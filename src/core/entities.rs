pub use ofdb_entities::{
    activity::*, address::*, category::*, comment::*, contact::*, email::*, event::*, geo::*,
    id::*, links::*, location::*, nonce::*, organization::*, password::*, place::*, rating::*,
    review::*, revision::*, subscription::*, tag::*, time::*, user::*,
};

#[cfg(test)]
pub trait Builder {
    type Build;
    fn build() -> Self::Build;
}

#[cfg(test)]
pub use self::place_builder::*;

#[cfg(test)]
pub mod place_builder {

    use super::*;
    use std::str::FromStr;

    pub struct PlaceBuild {
        place: Place,
    }

    impl PlaceBuild {
        pub fn id(mut self, id: &str) -> Self {
            self.place.id = id.into();
            self
        }
        pub fn revision(mut self, v: u64) -> Self {
            self.place.revision = v.into();
            self
        }
        pub fn title(mut self, title: &str) -> Self {
            self.place.title = title.into();
            self
        }
        pub fn description(mut self, desc: &str) -> Self {
            self.place.description = desc.into();
            self
        }
        pub fn pos(mut self, pos: MapPoint) -> Self {
            self.place.location.pos = pos;
            self
        }
        pub fn tags(mut self, tags: Vec<&str>) -> Self {
            self.place.tags = tags.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn license(mut self, license: &str) -> Self {
            self.place.license = license.into();
            self
        }
        pub fn image_url(mut self, image_url: Option<&str>) -> Self {
            self.place.links = match self.place.links {
                Some(mut links) => {
                    links.image = image_url.map(FromStr::from_str).transpose().unwrap();
                    Some(links)
                }
                None => {
                    if let Some(image_url) = image_url {
                        let links = Links {
                            image: Some(image_url.parse().unwrap()),
                            ..Default::default()
                        };
                        Some(links)
                    } else {
                        None
                    }
                }
            };
            self
        }
        pub fn image_link_url(mut self, image_link_url: Option<&str>) -> Self {
            self.place.links = match self.place.links {
                Some(mut links) => {
                    links.image_href = image_link_url.map(FromStr::from_str).transpose().unwrap();
                    Some(links)
                }
                None => {
                    if let Some(image_link_url) = image_link_url {
                        let links = Links {
                            image_href: Some(image_link_url.parse().unwrap()),
                            ..Default::default()
                        };
                        Some(links)
                    } else {
                        None
                    }
                }
            };
            self
        }
        pub fn finish(self) -> Place {
            self.place
        }
    }

    impl Builder for Place {
        type Build = PlaceBuild;
        fn build() -> PlaceBuild {
            PlaceBuild {
                place: Place {
                    id: Id::new(),
                    license: "".into(),
                    revision: Revision::initial(),
                    created: Activity::now(None),
                    title: "".into(),
                    description: "".into(),
                    location: Location {
                        pos: MapPoint::from_lat_lng_deg(0.0, 0.0),
                        address: None,
                    },
                    contact: None,
                    opening_hours: None,
                    links: None,
                    tags: vec![],
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
        pub fn state(mut self, x: &str) -> Self {
            self.addr.state = Some(x.into());
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
        assert_eq!(Address::build().state("x").finish().is_empty(), false);
    }
}
