use std::str::FromStr;
use time::Date;

use crate::{activity::*, contact::*, id::*, links::*, location::*, review::*, revision::*};

// Immutable part of a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceRoot {
    pub id: Id,
    pub license: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpeningHours(String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OpeningHoursParseError;

impl OpeningHours {
    pub const fn min_len() -> usize {
        4
    }
}

impl FromStr for OpeningHours {
    type Err = OpeningHoursParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.len() < Self::min_len() {
            return Err(OpeningHoursParseError);
        }
        Ok(Self(trimmed.to_string()))
    }
}

impl From<String> for OpeningHours {
    fn from(from: String) -> Self {
        let res = Self(from);
        debug_assert_eq!(Ok(&res), res.0.as_str().parse().as_ref());
        res
    }
}

impl From<OpeningHours> for String {
    fn from(from: OpeningHours) -> Self {
        from.0
    }
}

// Mutable part of a place.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceRevision {
    pub revision: Revision,
    pub created: Activity,
    pub title: String,
    pub description: String,
    pub location: Location,
    pub contact: Option<Contact>,
    pub opening_hours: Option<OpeningHours>,
    pub founded_on: Option<Date>,
    pub links: Option<Links>,
    pub tags: Vec<String>,
}

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
    pub opening_hours: Option<OpeningHours>,
    pub founded_on: Option<Date>,
    pub links: Option<Links>,
    pub tags: Vec<String>,
}

impl Place {
    pub fn strip_activity_details(self) -> Self {
        Self {
            created: self.created.anonymize(),
            ..self
        }
    }

    pub fn strip_contact_details(self) -> Self {
        Self {
            contact: None,
            ..self
        }
    }

    pub fn is_owned<'a>(&self, moderated_tags: impl IntoIterator<Item = &'a str>) -> bool {
        // Exclusive ownership of events is determined by the associated tags
        moderated_tags
            .into_iter()
            .any(|moderated_tag| self.tags.iter().any(|tag| tag == moderated_tag))
    }
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
                opening_hours,
                founded_on,
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
            opening_hours,
            founded_on,
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
            opening_hours,
            founded_on,
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
                opening_hours,
                founded_on,
                links,
                tags,
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceHistory {
    pub place: PlaceRoot,
    pub revisions: Vec<(PlaceRevision, Vec<ReviewStatusLog>)>,
}
