use crate::{activity::*, contact::*, id::*, links::*, location::*, review::*, revision::*};

// Immutable part of a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceRoot {
    pub id: Id,
    pub license: String,
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

    pub fn is_owned<'a>(&self, owned_tags: impl IntoIterator<Item = &'a str>) -> bool {
        // Exclusive ownership of events is determined by the associated tags
        owned_tags
            .into_iter()
            .any(|owned_tag| self.tags.iter().any(|tag| tag == owned_tag))
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlaceHistory {
    pub place: PlaceRoot,
    pub revisions: Vec<(PlaceRevision, Vec<ReviewStatusLog>)>,
}
