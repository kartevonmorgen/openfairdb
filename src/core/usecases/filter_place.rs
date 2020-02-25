use crate::core::prelude::*;

pub fn filter_place<'a>(place: Place, owned_tags: impl IntoIterator<Item = &'a str>) -> Place {
    if place.is_owned(owned_tags) {
        place
    } else {
        place.strip_activity_details()
    }
}
