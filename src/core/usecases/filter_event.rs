use crate::core::prelude::*;

pub fn filter_event<'a>(event: Event, owned_tags: impl IntoIterator<Item = &'a str>) -> Event {
    if event.is_owned(owned_tags) {
        event
    } else {
        event.strip_activity_details()
    }
}
