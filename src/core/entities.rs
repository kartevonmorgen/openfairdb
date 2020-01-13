use crate::core::util::{geo::MapBbox, nonce::Nonce};
use anyhow::{bail, format_err, Result as Fallible};
pub use ofdb_core::{
    time::{Timestamp, TimestampMs},
    *,
};

// Convenience type that merges the tuple (PlaceRoot, PlaceRevision)
// into a single, flat struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub id: Id,
    pub license: String,
    pub revision: Revision,
    pub created: Activity,
    pub title: String,
    pub description: String,
    pub location: Location,
    pub contact: Option<Contact>,
    pub links: Option<Links>,
    pub tags: Vec<String>,
}

impl From<(PlaceRoot, PlaceRevision)> for Place {
    fn from(from: (PlaceRoot, PlaceRevision)) -> Self {
        let (
            PlaceRoot { id, license },
            PlaceRevision {
                revision,
                created,
                title,
                description,
                location,
                contact,
                links,
                tags,
            },
        ) = from;
        Self {
            id,
            license,
            revision,
            created,
            title,
            description,
            location,
            contact,
            links,
            tags,
        }
    }
}

impl From<Place> for (PlaceRoot, PlaceRevision) {
    fn from(from: Place) -> Self {
        let Place {
            id,
            license,
            revision,
            created,
            title,
            description,
            location,
            contact,
            links,
            tags,
        } = from;
        (
            PlaceRoot { id, license },
            PlaceRevision {
                revision,
                created,
                title,
                description,
                location,
                contact,
                links,
                tags,
            },
        )
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
pub struct Rating {
    pub id          : Id,
    pub place_id    : Id,
    // TODO: Convert time stamps from second to millisecond precision?
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
    pub id         : Id,
    pub user_email : String,
    pub bbox       : MapBbox,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UserToken {
    pub email_nonce: EmailNonce,
    // TODO: Convert time stamps from second to millisecond precision?
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
pub use self::place_builder::*;

#[cfg(test)]
pub mod place_builder {

    use super::*;
    use ofdb_core::geo::MapPoint;
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
                    links: None,
                    tags: vec![],
                },
            }
        }
    }
}
