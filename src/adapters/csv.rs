use crate::core::{entities::*, util::time::Timestamp};

#[derive(Debug, Serialize)]
pub struct CsvRecord {
    pub id: String,
    pub created_at: i64,
    pub created_by: Option<String>,
    pub version: u64,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lng: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub homepage: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub opening_hours: Option<String>,
    pub founded_on: Option<String>,
    pub categories: String,
    pub tags: String,
    pub license: String,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    pub avg_rating: f64,
}

impl From<(Place, Vec<Category>, AvgRatingValue)> for CsvRecord {
    fn from(from: (Place, Vec<Category>, AvgRatingValue)) -> Self {
        let (place, categories, avg_rating) = from;

        let Place {
            id,
            license,
            revision,
            created:
                Activity {
                    at: created_at,
                    by: created_by,
                },
            title,
            description,
            location,
            links,
            tags,
            contact,
            opening_hours,
            founded_on,
            ..
        } = place;

        let Location { pos, address } = location;

        let address = address.unwrap_or_default();

        let Address {
            street,
            zip,
            city,
            country,
            state,
        } = address;

        let (homepage_url, image_url, image_link_url) = if let Some(links) = links {
            (links.homepage, links.image, links.image_href)
        } else {
            (None, None, None)
        };

        let categories = categories
            .into_iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
            .join(",");

        let (contact_name, contact_email, contact_phone) = if let Some(contact) = contact {
            let Contact { name, phone, email } = contact;
            (name, email, phone)
        } else {
            (None, None, None)
        };

        CsvRecord {
            id: id.into(),
            created_at: created_at.into_seconds(),
            created_by: created_by.map(Into::into),
            version: revision.into(),
            title,
            description,
            lat: pos.lat().to_deg(),
            lng: pos.lng().to_deg(),
            street,
            zip,
            city,
            country,
            state,
            homepage: homepage_url.map(Url::into_string),
            contact_name,
            contact_phone,
            contact_email: contact_email.map(Into::into),
            opening_hours: opening_hours.map(Into::into),
            founded_on: founded_on.as_ref().map(ToString::to_string),
            license,
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
            categories,
            tags: tags.join(","),
            avg_rating: avg_rating.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EventRecord {
    pub id: String,
    pub created_by: Option<String>,
    pub organizer: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub start: i64,
    pub end: Option<i64>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    pub tags: String,
}

impl From<Event> for EventRecord {
    fn from(from: Event) -> Self {
        let Event {
            id,
            created_by,
            title,
            description,
            start,
            end,
            location,
            contact,
            homepage,
            image_url,
            image_link_url,
            tags,
            ..
        } = from;

        let (pos, address) = location.map_or((None, None), |l| {
            let Location { pos, address } = l;
            if pos.is_valid() {
                (Some(pos), address)
            } else {
                (None, address)
            }
        });

        let (lat, lng) = pos.map_or((None, None), |p| {
            (Some(p.lat().to_deg()), Some(p.lng().to_deg()))
        });

        let address = address.unwrap_or_default();
        let Address {
            street,
            zip,
            city,
            country,
            state,
        } = address;

        let Contact {
            name: organizer,
            email,
            phone,
        } = contact.unwrap_or_default();

        Self {
            id: id.into(),
            created_by,
            title,
            description,
            start: Timestamp::from(start).into_seconds(),
            end: end.map(|end| Timestamp::from(end).into_seconds()),
            lat,
            lng,
            street,
            zip,
            city,
            country,
            state,
            organizer,
            email: email.map(Into::into),
            phone,
            homepage: homepage.map(Url::into_string),
            image_url: image_url.map(Url::into_string),
            image_link_url: image_link_url.map(Url::into_string),
            tags: tags.join(","),
        }
    }
}
