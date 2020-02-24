use crate::core::prelude::*;

fn is_place_owned(place: &Place, owned_tags: &[String]) -> bool {
    // Exclusive ownership of places is determined by the associated
    // tags.
    owned_tags
        .iter()
        .any(|ref tag| place.tags.iter().any(|ref e_tag| e_tag == tag))
}

pub fn export_place(place: Place, role: Role, owned_tags: &[String]) -> Place {
    if role >= Role::Admin {
        // Admin sees everything even if no owned tags are provided
        place
    } else {
        // Contact details are only visible for scouts and admins
        if role >= Role::Scout {
            let owned = is_place_owned(&place, owned_tags);
            if owned {
                // Include created.by
                place
            } else {
                // Exclude created.by
                Place {
                    created: Activity {
                        by: None,
                        ..place.created
                    },
                    ..place
                }
            }
        } else {
            // Neither contact details nor created.by should be exported
            Place {
                created: Activity {
                    by: None,
                    ..place.created
                },
                contact: None,
                ..place
            }
        }
    }
}
