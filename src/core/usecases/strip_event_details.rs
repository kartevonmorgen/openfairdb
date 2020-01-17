use crate::core::prelude::*;

fn is_event_owned(event: &Event, owned_tags: &[String]) -> bool {
    // Exclusive ownership of events is determined by the associated
    // tags.
    owned_tags
        .iter()
        .any(|ref tag| event.tags.iter().any(|ref e_tag| e_tag == tag))
}

pub fn strip_event_details_on_search(
    event_iter: impl Iterator<Item = Event>,
    owned_tags: Vec<String>,
) -> impl Iterator<Item = Event> {
    event_iter.map(move |e| {
        // Hide the created_by e-mail address if the event is not owned.
        if is_event_owned(&e, &owned_tags) {
            // Include created_by
            e
        } else {
            // Exclude created_by
            Event {
                created_by: None,
                ..e
            }
        }
    })
}

pub fn strip_event_details_on_export(
    event_iter: impl Iterator<Item = Event>,
    role: Role,
    owned_tags: Vec<String>,
) -> impl Iterator<Item = Event> {
    event_iter.map(move |e| {
        // Hide contact data for everyone except admins on batch exports
        let owned = is_event_owned(&e, &owned_tags);
        if role >= Role::Scout || owned {
            // Include contact details
            if owned {
                // Include created_by
                e
            } else {
                // Exclude created_by
                Event {
                    created_by: None,
                    ..e
                }
            }
        } else {
            // Exclude contact details
            Event { contact: None, ..e }
        }
    })
}
