use crate::usecases::{filter_place::filter_place, prelude::*};

pub fn export_place<'a>(
    place: Place,
    role: Role,
    moderated_tags: impl IntoIterator<Item = &'a str>,
) -> Place {
    if role < Role::Admin {
        let place = filter_place(place, moderated_tags);
        if role < Role::Scout {
            place.strip_contact_details()
        } else {
            place
        }
    } else {
        place
    }
}
