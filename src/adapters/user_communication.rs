use crate::core::entities::*;
use crate::core::usecases::{NewEntry, UpdateEntry};

pub fn email_confirmation_email(u_id: &str) -> String {
    format!(
        "Na du Weltverbesserer*,\nwir freuen uns dass du bei der Karte von morgen mit dabei bist!\n\nBitte bestätige deine Email-Adresse hier:\nhttps://kartevonmorgen.org/#/?confirm_email={}.\n\neuphorische Grüße\ndas Karte von morgen-Team",
        u_id
    )
}

pub fn new_entry_email(e: &NewEntry, id: &str, categories: &[String]) -> String {
    let intro_sentence = "ein neuer Eintrag auf der Karte von morgen wurde erstellt";

    //TODO: check fields
    let address = Some(Address {
        street: e.street.clone(),
        zip: e.zip.clone(),
        city: e.city.clone(),
        country: e.country.clone(),
    });

    let contact = Some(Contact {
        email: e.email.clone(),
        telephone: e.telephone.clone(),
    });

    let entry = Entry {
        id: id.into(),
        osm_node: None,
        title: e.title.clone(),
        description: e.description.clone(),
        homepage: e.homepage.clone(),
        tags: e.tags.clone(),
        categories: e.categories.clone(),
        location: Location {
            lat: 0.0,
            lng: 0.0,
            address,
        },
        contact,
        created: 0,
        version: 0,
        license: None,
        image_url: None,
        image_link_url: None,
    };
    entry_email(&entry, categories, &e.tags, intro_sentence)
}

//TODO: calc diff
pub fn changed_entry_email(e: &UpdateEntry, categories: &[String]) -> String {
    let intro_sentence = "folgender Eintrag der Karte von morgen wurde verändert";

    let address = Some(Address {
        street: e.street.clone(),
        zip: e.zip.clone(),
        city: e.city.clone(),
        country: e.country.clone(),
    });

    let contact = Some(Contact {
        email: e.email.clone(),
        telephone: e.telephone.clone(),
    });

    let entry = Entry {
        id: e.id.clone(),
        osm_node: e.osm_node,
        title: e.title.clone(),
        description: e.description.clone(),
        homepage: e.homepage.clone(),
        tags: e.tags.clone(),
        categories: e.categories.clone(),
        location: Location {
            lat: 0.0,
            lng: 0.0,
            address,
        },
        contact,
        created: 0,
        version: 0,
        license: None,
        image_url: None,
        image_link_url: None,
    };
    entry_email(&entry, categories, &e.tags, intro_sentence)
}

pub fn entry_email(
    e: &Entry,
    categories: &[String],
    tags: &[String],
    intro_sentence: &str,
) -> String {
    let category = if !categories.is_empty() {
        categories[0].clone()
    } else {
        "".to_string()
    };

    let Address {
        street,
        zip,
        city,
        country,
    } = e.location.clone().address.unwrap_or_else(|| Address {
        street: None,
        zip: None,
        city: None,
        country: None,
    });

    let address = vec![
        street.unwrap_or_else(|| "".into()),
        vec![
            zip.unwrap_or_else(|| "".into()),
            city.unwrap_or_else(|| "".into()),
        ]
        .join(" "),
        country.unwrap_or_else(|| "".into()),
    ]
    .join(", ");

    let Contact { email, telephone } = e.contact.clone().unwrap_or_else(|| Contact {
        email: None,
        telephone: None,
    });

    format!(
        "Hallo,
{introSentence}:\n
{title} ({category})
{description}\n
    Tags: {tags}
    Adresse: {address}
    Webseite: {homepage}
    Email-Adresse: {email}
    Telefon: {telephone}\n
Eintrag anschauen oder bearbeiten:
https://kartevonmorgen.org/#/?entry={id}\n
Du kannst dein Abonnement des Kartenbereichs abbestellen indem du dich auf https://kartevonmorgen.org einloggst.\n
euphorische Grüße
das Karte von morgen-Team",
        introSentence = intro_sentence,
        title = &e.title,
        id = &e.id,
        description = &e.description,
        address = address,
        email = email.unwrap_or_else(||"".into()),
        telephone = telephone.unwrap_or_else(||"".into()),
        homepage = e.homepage.clone().unwrap_or_else(||"".into()),
        category = category,
        tags = tags.join(", ")
    )
}
