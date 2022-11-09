use askama::Template;
use ofdb_entities::{address::*, contact::*, email::*, event::*, place::*, url::*};
use time::{format_description::FormatItem, macros::format_description};

mod email_reminder_formatter;
pub use email_reminder_formatter::*;

const DATE_TIME_FORMAT: &[FormatItem] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

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
            ref state,
        } = address;
        [
            street.as_deref().unwrap_or(""),
            &[zip.as_deref().unwrap_or(""), city.as_deref().unwrap_or("")].join(" "),
            country.as_deref().unwrap_or(""),
            state.as_deref().unwrap_or(""),
        ]
        .join(", ")
    } else {
        Default::default()
    }
}

#[derive(Template)]
#[template(path = "email_user_registration/subject_DE.txt")]
struct EmailUserRegistrationSubjectTemplate;

#[derive(Template)]
#[template(path = "email_user_registration/body_DE.txt")]
struct EmailUserRegistrationBodyTemplate<'a> {
    url: &'a str,
}

pub fn user_registration_email(url: &str) -> EmailContent {
    let subject = EmailUserRegistrationSubjectTemplate.render().unwrap();
    let body = EmailUserRegistrationBodyTemplate { url }.render().unwrap();
    EmailContent { subject, body }
}

#[derive(Template)]
#[template(path = "email_reset_password/subject_DE.txt")]
struct EmailUserResetPasswordSubjectTemplate;

#[derive(Template)]
#[template(path = "email_reset_password/body_DE.txt")]
struct EmailUserResetPasswordBodyTemplate<'a> {
    url: &'a str,
}

pub fn user_reset_password_email(url: &str) -> EmailContent {
    let subject = EmailUserResetPasswordSubjectTemplate.render().unwrap();
    let body = EmailUserResetPasswordBodyTemplate { url }.render().unwrap();
    EmailContent { subject, body }
}

pub fn place_created_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = subject_entry_created(&place.title);
    let body = place_email(place, category_names, &EventType::Created);
    EmailContent { subject, body }
}

//TODO: calc diff
pub fn place_updated_email(place: &Place, category_names: &[String]) -> EmailContent {
    let subject = subject_entry_updated(&place.title);
    let body = place_email(place, category_names, &EventType::Updated);
    EmailContent { subject, body }
}

fn place_email(place: &Place, category_names: &[String], event_type: &EventType) -> String {
    let category = if !category_names.is_empty() {
        &category_names[0]
    } else {
        ""
    };
    let Contact { email, phone, .. } = place.contact.clone().unwrap_or_default();

    let id = place.id.as_str();
    let title = &place.title;
    let description = &place.description;
    let address_line = &address_line(place.location.address.as_ref());
    let email = &email.map(|e| e.to_string()).unwrap_or_default();
    let phone = &phone.unwrap_or_default();
    let homepage = place
        .links
        .as_ref()
        .and_then(|l| l.homepage.as_ref())
        .map(Url::as_str)
        .unwrap_or_else(|| "");
    let tags = &place.tags.join(", ");

    PlaceEmailTemplate {
        event_type,
        title,
        category,
        description,
        tags,
        address_line,
        homepage,
        email,
        phone,
        id,
    }
    .render()
    .unwrap()
}

#[derive(Template)]
#[template(path = "place_email_DE.txt")]
struct PlaceEmailTemplate<'a> {
    event_type: &'a EventType,
    title: &'a str,
    category: &'a str,
    description: &'a str,
    tags: &'a str,
    address_line: &'a str,
    homepage: &'a str,
    email: &'a str,
    phone: &'a str,
    id: &'a str,
}

pub fn event_created_email(event: &Event) -> EmailContent {
    let subject = subject_entry_created(&event.title);
    let body = event_email(event, &EventType::Created);
    EmailContent { subject, body }
}

//TODO: calc diff
pub fn event_updated_email(event: &Event) -> EmailContent {
    let subject = subject_entry_updated(&event.title);
    let body = event_email(event, &EventType::Updated);
    EmailContent { subject, body }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum EventType {
    Created,
    Updated,
}

fn event_email(event: &Event, event_type: &EventType) -> String {
    let Contact { email, phone, .. } = event.contact.clone().unwrap_or_default();
    let category = "Event";
    let id = &event.id.as_str();
    let title = &event.title;
    let start = &event.start.format(DATE_TIME_FORMAT);
    let end = &event
        .end
        .map(|end| end.format(DATE_TIME_FORMAT))
        .unwrap_or_default();
    let description = event.description.as_deref().unwrap_or("");
    let organizer = event.organizer().map(String::as_str).unwrap_or("");
    let address_line = &address_line(event.location.as_ref().and_then(|l| l.address.as_ref()));
    let email = &email.map(|e| e.to_string()).unwrap_or_default();
    let phone = &phone.unwrap_or_default();
    let homepage = event.homepage.as_ref().map(Url::as_str).unwrap_or("");
    let tags = &event.tags.join(", ");

    EventEmailTemplate {
        event_type,
        title,
        category,
        description,
        start,
        end,
        tags,
        organizer,
        address_line,
        homepage,
        email,
        phone,
        id,
    }
    .render()
    .unwrap()
}

#[derive(Template)]
#[template(path = "event_email_DE.txt")]
struct EventEmailTemplate<'a> {
    event_type: &'a EventType,
    title: &'a str,
    category: &'a str,
    description: &'a str,
    start: &'a str,
    end: &'a str,
    tags: &'a str,
    organizer: &'a str,
    address_line: &'a str,
    homepage: &'a str,
    email: &'a str,
    phone: &'a str,
    id: &'a str,
}

#[cfg(test)]
mod tests {
    use ofdb_entities::{activity::*, geo::*, links::*, location::*, revision::*, time::*};

    use super::*;

    // To verify the formatting manually run these tests and examine
    // the output on stdout:
    //
    // ```sh
    // cargo test --tests user_communication -- --nocapture
    // ```

    fn print_email(email: &EmailContent) {
        let EmailContent { subject, body } = email;
        // 72 column ruler
        println!("========================================================================");
        println!("{subject}");
        println!("------------------------------------------------------------------------");
        println!("{body}");
        println!("========================================================================");
    }

    const OUTRO_HINT_DE: &str = include_str!("templates/outro_hints_DE.txt");
    const INTRO_ENTRY_CREATED_DE: &str = include_str!("templates/intro_entry_created_DE.txt");
    const INTRO_ENTRY_UPDATED_DE: &str = include_str!("templates/intro_entry_updated_DE.txt");

    fn new_place() -> Place {
        Place {
            id: "<id>".into(),
            license: "<license>".into(),
            revision: Revision::initial(),
            created: Activity {
                at: Timestamp::now(),
                by: Some("created_by@example.com".parse().unwrap()),
            },
            title: "<title>".into(),
            description: "<description>".into(),
            location: Location {
                pos: MapPoint::try_from_lat_lng_deg(42.27, -7.97).unwrap(),
                address: Some(Address {
                    street: Some("<street>".into()),
                    zip: Some("<zip>".into()),
                    city: Some("<city>".into()),
                    country: Some("<country>".into()),
                    state: Some("<state>".into()),
                }),
            },
            contact: Some(Contact {
                name: Some("<name>".into()),
                email: Some(EmailAddress::new_unchecked("<email>".into())),
                phone: Some("<phone>".into()),
            }),
            opening_hours: Some("24/7".parse().unwrap()),
            founded_on: Some(
                time::Date::from_calendar_date(1945, time::Month::October, 24).unwrap(),
            ),
            links: Some(Links {
                homepage: Some("https://kartevonmorgen.org".parse().unwrap()),
                ..Default::default()
            }),
            tags: vec!["<tag1>".into(), "<tag2>".into()],
        }
    }

    fn new_event() -> Event {
        Event {
            id: "<id>".into(),
            created_by: Some("created_by@example.com".parse().unwrap()),
            archived: None,
            start: Timestamp::now(),
            end: None,
            registration: None,
            title: "<title>".into(),
            description: Some("<description>".into()),
            location: Some(Location {
                pos: MapPoint::try_from_lat_lng_deg(42.27, -7.97).unwrap(),
                address: Some(Address {
                    street: Some("<street>".into()),
                    zip: Some("<zip>".into()),
                    city: Some("<city>".into()),
                    country: Some("<country>".into()),
                    state: Some("<state>".into()),
                }),
            }),
            contact: Some(Contact {
                name: Some("<organizer>".into()),
                email: Some(EmailAddress::new_unchecked("<email>".into())),
                phone: Some("<phone>".into()),
            }),
            homepage: Some("https://kartevonmorgen.org".parse().unwrap()),
            image_url: None,
            image_link_url: None,
            tags: vec!["<tag1>".into(), "<tag2>".into()],
        }
    }

    #[test]
    fn print_user_registration_email() {
        let url = "https://kartevonmorgen.org/confirm-email/";
        let email = user_registration_email(url);
        assert!(email.body.contains(OUTRO_HINT_DE));
        assert!(email.body.contains(url));
        print_email(&email);
    }

    #[test]
    fn print_user_reset_password_email() {
        let url = "https://kartevonmorgen.org/reset-password/";
        let email = user_reset_password_email(url);
        assert!(email.body.contains(url));
        print_email(&email);
    }

    #[test]
    fn print_place_created_email() {
        let place = new_place();
        let email = place_created_email(&place, &["<category>".into()]);
        assert!(email.body.contains(INTRO_ENTRY_CREATED_DE));
        assert!(email.body.contains(OUTRO_HINT_DE));
        assert!(email.body.contains(place.id.as_str()));
        assert!(email.body.contains(&place.title));
        print_email(&email);
    }

    #[test]
    fn print_place_updated_email() {
        let place = new_place();
        let email = place_updated_email(&place, &["<category>".into()]);
        assert!(email.body.contains(INTRO_ENTRY_UPDATED_DE));
        assert!(email.body.contains(OUTRO_HINT_DE));
        assert!(email.body.contains(place.id.as_str()));
        assert!(email.body.contains(&place.title));
        print_email(&email);
    }

    #[test]
    fn print_event_created_email() {
        let event = new_event();
        let email = event_created_email(&event);
        assert!(email.body.contains(INTRO_ENTRY_CREATED_DE));
        assert!(email.body.contains(OUTRO_HINT_DE));
        assert!(email.body.contains(event.id.as_str()));
        assert!(email.body.contains(&event.title));
        print_email(&email);
    }

    #[test]
    fn print_event_updated_email() {
        let event = new_event();
        let email = event_updated_email(&event);
        assert!(email.body.contains(INTRO_ENTRY_UPDATED_DE));
        assert!(email.body.contains(OUTRO_HINT_DE));
        assert!(email.body.contains(event.id.as_str()));
        assert!(email.body.contains(&event.title));
        print_email(&email);
    }
}
