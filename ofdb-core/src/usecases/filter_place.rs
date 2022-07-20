use super::prelude::*;

pub fn filter_place<'a>(place: Place, moderated_tags: impl IntoIterator<Item = &'a str>) -> Place {
    if place.is_owned(moderated_tags) {
        place
    } else {
        place.strip_activity_details()
    }
}
