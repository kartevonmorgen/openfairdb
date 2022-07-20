use super::prelude::*;

pub fn filter_event<'a>(event: Event, moderated_tags: impl IntoIterator<Item = &'a str>) -> Event {
    if event.is_owned(moderated_tags) {
        event
    } else {
        event.strip_activity_details()
    }
}
