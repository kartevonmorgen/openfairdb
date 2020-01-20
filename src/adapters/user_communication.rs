use crate::core::prelude::*;

use url::Url;

pub struct EmailContent {
    pub subject: String,
    pub body: String,
}

const DATE_TIME_FORMAT: &str = "%Y.%m.%d %H:%M:%S";

const INTRO_ENTRY_CREATED: &str = "ein neuer Eintrag auf der Karte von morgen wurde erstellt";

const INTRO_ENTRY_UPDATED: &str = "folgender Eintrag auf der Karte von morgen wurde verändert";

const OUTRO_HINT: &str = "Weitere Hinweise und Tipps zur Nutzung, z.B. wie du interaktive Karten
per <iframe> auf deiner Webseite einbettest oder Papierkarten erstellst,
findest du hier: https://blog.vonmorgen.org";

fn subject_entry_created(entry_title: &str) -> String {
    format!("Kvm - neuer Eintrag: {}", entry_title)
}

fn subject_entry_updated(entry_title: &str) -> String {
    format!("Kvm - Eintrag verändert: {}", entry_title)
}

fn address_line(address: Option<&Address>) -> String {
    if let Some(address) = address {
        let Address {
            ref street,
            ref zip,
            ref city,
            ref country,
        } = address;
        [
            street.as_ref().map(String::as_str).unwrap_or(""),
            &[
                zip.as_ref().map(String::as_str).unwrap_or(""),
                city.as_ref().map(String::as_str).unwrap_or(""),
            ]
            .join(" "),
            country.as_ref().map(String::as_str).unwrap_or(""),
        ]
        .join(", ")
    } else {
        Default::default()
    }
}

pub fn user_registration_email(url: &str) -> EmailContent {
    let subject = "Karte von morgen: Bitte bestätige deine Email-Adresse".into();
    let body = format!(
        "Na du Weltverbesserer*,\n
wir freuen uns, dass du bei der Karte von morgen mit dabei bist!\n\n
Bitte bestätige deine Email-Adresse hier:\n
{url}\n\n
euphorische Grüße,\n
das Karte von morgen-Team\n
{outro_text}",
        url = url,
        outro_text = OUTRO_HINT,
    );
    EmailContent { subject, body }
}

pub fn user_reset_password_email(url: &str) -> EmailContent {
    let subject = "Karte von morgen: Passwort zurücksetzen".into();
    let body = format!(
        "Na du Weltverbesserer*,\n
hast du uns kürzlich gebeten dein Passwort zurücksetzen?\n\n
Bitte folge zur Eingabe eines neuen Passworts diesem Link:\n
{url}\n\n
euphorische Grüße,\n
das Karte von morgen-Team",
        url = url,
    );
    EmailContent { subject, body }
}

pub fn place_created_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = subject_entry_created(&place.title);
    let body = place_email(place, category_names, INTRO_ENTRY_CREATED);
    EmailContent { subject, body }
}

//TODO: calc diff
pub fn place_updated_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = subject_entry_updated(&place.title);
    let body = place_email(place, category_names, INTRO_ENTRY_UPDATED);
    EmailContent { subject, body }
}

fn place_email(place: &Place, category_names: &[String], intro_sentence: &str) -> String {
    let category = if !category_names.is_empty() {
        category_names[0].clone()
    } else {
        "".to_string()
    };

    let Contact { email, phone } = place.contact.clone().unwrap_or_else(|| Contact {
        email: None,
        phone: None,
    });

    format!(
        "Hallo,\n
{intro_sentence}:\n
{title} ({category})
{description}\n
    Tags: {tags}
    Adresse: {address_line}
    Webseite: {homepage}
    Email-Adresse: {email}
    Telefon: {phone}\n
Eintrag anschauen oder bearbeiten:
https://kartevonmorgen.org/#/?entry={id}\n
Du kannst dein Abonnement des Kartenbereichs abbestellen,
indem du dich auf https://kartevonmorgen.org einloggst.\n
euphorische Grüße,\n
das Karte von morgen-Team\n
{outro_text}",
        intro_sentence = intro_sentence,
        outro_text = OUTRO_HINT,
        id = &place.id,
        title = &place.title,
        description = &place.description,
        address_line = address_line(place.location.address.as_ref()),
        email = email.map(|e| e.to_string()).unwrap_or_default(),
        phone = phone.unwrap_or_default(),
        homepage = place
            .links
            .as_ref()
            .and_then(|l| l.homepage.as_ref())
            .map(Url::as_str)
            .unwrap_or_else(|| ""),
        category = category,
        tags = place.tags.join(", ")
    )
}

pub fn event_created_email(event: &Event) -> EmailContent {
    let subject = subject_entry_created(&event.title);
    let body = event_email(event, INTRO_ENTRY_CREATED);
    EmailContent { subject, body }
}

//TODO: calc diff
pub fn event_updated_email(event: &Event) -> EmailContent {
    let subject = subject_entry_updated(&event.title);
    let body = event_email(event, INTRO_ENTRY_UPDATED);
    EmailContent { subject, body }
}

fn event_email(event: &Event, intro_sentence: &str) -> String {
    let Contact { email, phone } = event.contact.clone().unwrap_or_else(|| Contact {
        email: None,
        phone: None,
    });

    format!(
        "Hallo,\n
{intro_sentence}:\n
{title} ({category})
{description}\n
    Beginn: {start}
    Ende: {end}
    Tags: {tags}
    Veranstalter: {organizer}
    Adresse: {address_line}
    Webseite: {homepage}
    Email-Adresse: {email}
    Telefon: {phone}\n
Eintrag anschauen oder bearbeiten:
https://kartevonmorgen.org/#/?entry={id}\n
Du kannst dein Abonnement des Kartenbereichs abbestellen,
indem du dich auf https://kartevonmorgen.org einloggst.\n
euphorische Grüße,\n
das Karte von morgen-Team\n
{outro_text}",
        intro_sentence = intro_sentence,
        outro_text = OUTRO_HINT,
        category = "Event",
        id = &event.id,
        title = &event.title,
        start = event.start.format(DATE_TIME_FORMAT),
        end = event
            .end
            .map(|end| end.format(DATE_TIME_FORMAT).to_string())
            .unwrap_or_default(),
        description = event.description.as_ref().map(String::as_str).unwrap_or(""),
        organizer = event.organizer.as_ref().map(String::as_str).unwrap_or(""),
        address_line = address_line(event.location.as_ref().and_then(|l| l.address.as_ref())),
        email = email.map(|e| e.to_string()).unwrap_or_default(),
        phone = phone.unwrap_or_default(),
        homepage = event.homepage.as_ref().map(Url::as_str).unwrap_or(""),
        tags = event.tags.join(", ")
    )
}
