use crate::core::prelude::*;

use url::Url;

pub struct EmailContent {
    pub subject: String,
    pub body: String,
}

pub fn user_registration_email(url: &str) -> EmailContent {
    let subject = "Karte von morgen: Bitte bestätige deine Email-Adresse".into();
    let body = format!(
        "Na du Weltverbesserer*,\nwir freuen uns dass du bei der Karte von morgen mit dabei bist!\n\nBitte bestätige deine Email-Adresse hier:\n{}\n\neuphorische Grüße\ndas Karte von morgen-Team",
        url
    );
    EmailContent { subject, body }
}

pub fn user_reset_password_email(url: &str) -> EmailContent {
    let subject = "Karte von morgen: Passwort zurücksetzen".into();
    let body = format!(
        "Na du Weltverbesserer*,\nhast Du uns kürzlich gebeten Dein Passwort zurücksetzen?\n\nBitte folge zur Eingabe eines neuen Passworts diesem Link:\n{}\n\neuphorische Grüße\ndas Karte von morgen-Team",
        url,
    );
    EmailContent { subject, body }
}

pub fn entry_added_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = format!("Karte von morgen - neuer Eintrag: {}", place.title);
    let intro_sentence = "ein neuer Eintrag auf der Karte von morgen wurde erstellt";
    let body = entry_email(place, category_names, intro_sentence);
    EmailContent { subject, body }
}

//TODO: calc diff
pub fn entry_changed_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = format!("Karte von morgen - Eintrag verändert: {}", place.title);
    let intro_sentence = "folgender Eintrag der Karte von morgen wurde verändert";
    let body = entry_email(place, category_names, intro_sentence);
    EmailContent { subject, body }
}

fn entry_email(place: &Place, category_names: &[String], intro_sentence: &str) -> String {
    let category = if !category_names.is_empty() {
        category_names[0].clone()
    } else {
        "".to_string()
    };

    let Address {
        street,
        zip,
        city,
        country,
    } = place.location.clone().address.unwrap_or_else(|| Address {
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

    let Contact { email, phone } = place.contact.clone().unwrap_or_else(|| Contact {
        email: None,
        phone: None,
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
    Telefon: {phone}\n
Eintrag anschauen oder bearbeiten:
https://kartevonmorgen.org/#/?entry={id}\n
Du kannst dein Abonnement des Kartenbereichs abbestellen indem du dich auf https://kartevonmorgen.org einloggst.\n
euphorische Grüße
das Karte von morgen-Team",
        introSentence = intro_sentence,
        title = &place.title,
        id = &place.uid,
        description = &place.description,
        address = address,
        email = email.map(|e| e.to_string()).unwrap_or_default(),
        phone = phone.unwrap_or_default(),
        homepage = place.links.as_ref().and_then(|l| l.homepage.as_ref()).map(Url::as_str).unwrap_or_else(|| ""),
        category = category,
        tags = place.tags.join(", ")
    )
}
