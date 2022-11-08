use askama::Template;
use ofdb_core::usecases::{EmailReminderFormatter, Reminder};
use ofdb_entities::{email::EmailContent, place::Place};

#[derive(Default)]
pub struct ReminderFormatter; // TODO: support different languages

impl EmailReminderFormatter for ReminderFormatter {
    fn format_email(&self, r: &Reminder) -> EmailContent {
        let Place {
            id,
            title,
            description,
            ..
        } = &r.place;
        let subject = EmailReminderScoutsSubjectTemplate { title }
            .render()
            .unwrap();
        let body = EmailReminderScoutsBodyTemplate {
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
    title: &'a str,
    description: &'a str,
    id: &'a str,
}
