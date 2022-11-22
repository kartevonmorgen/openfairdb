use askama::Template;
use ofdb_core::usecases::{EmailReminderFormatter, Reminder};
use ofdb_entities::{email::EmailContent, place::Place};
use time::{format_description::FormatItem, macros::format_description};

#[derive(Default)]
pub struct ReminderFormatter; // TODO: support different languages

const DATE_FORMAT_DE: &[FormatItem] = format_description!("[day].[month].[year]");

impl EmailReminderFormatter for ReminderFormatter {
    fn format_email(&self, r: &Reminder) -> EmailContent {
        let Place {
            id,
            title,
            description,
            ..
        } = &r.place;
        let last_change = &r.last_change.format(DATE_FORMAT_DE); // TODO: support i18n
        let subject = EmailReminderScoutsSubjectTemplate { title }
            .render()
            .unwrap();
        let body = EmailReminderScoutsBodyTemplate {
            last_change,
            title,
            description,
            id: id.as_str(),
        }
        .render()
        .unwrap();
        EmailContent { subject, body }
    }
}

#[derive(Template)]
#[template(path = "email_reminder_scouts/subject_DE.txt")]
struct EmailReminderScoutsSubjectTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "email_reminder_scouts/body_DE.txt")]
struct EmailReminderScoutsBodyTemplate<'a> {
    last_change: &'a str,
    title: &'a str,
    description: &'a str,
    id: &'a str,
}
