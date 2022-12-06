use askama::Template;
use ofdb_core::usecases::{EmailReminderFormatter, RecipientRole, Reminder};
use ofdb_entities::{email::EmailContent, place::Place};
use time::{format_description::FormatItem, macros::format_description};

// TODO: support different languages
pub struct ReminderFormatter {
    recipient_role: RecipientRole,
}

impl ReminderFormatter {
    pub const fn new(recipient_role: RecipientRole) -> Self {
        Self { recipient_role }
    }
}

const DATE_FORMAT_DE: &[FormatItem] = format_description!("[day].[month].[year]");

impl EmailReminderFormatter for ReminderFormatter {
    fn format_email(&self, r: &Reminder) -> EmailContent {
        let Place {
            id,
            title,
            description,
            tags,
            ..
        } = &r.place;
        let last_change = &r.last_change.format(DATE_FORMAT_DE); // TODO: support i18n
        let subject = match self.recipient_role {
            RecipientRole::Scout => EmailReminderScoutsSubjectTemplate { title }.render(),
            RecipientRole::Owner => EmailReminderOwnerSubjectTemplate { title }.render(),
        }
        .unwrap();
        // TODO: inject base URL
        let entry_url = &format!("https://kartevonmorgen.org/#/?entry={id}");

        let tags = &tags.join(",");
        let body = match self.recipient_role {
            RecipientRole::Scout => EmailReminderScoutsBodyTemplate {
                last_change,
                title,
                description,
                tags,
                entry_url,
            }
            .render(),
            RecipientRole::Owner => EmailReminderOwnerBodyTemplate {
                last_change,
                title,
                description,
                tags,
                entry_url,
            }
            .render(),
        }
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
    tags: &'a str,
    entry_url: &'a str,
}

#[derive(Template)]
#[template(path = "email_reminder_owner/subject_DE.txt")]
struct EmailReminderOwnerSubjectTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "email_reminder_owner/body_DE.txt")]
struct EmailReminderOwnerBodyTemplate<'a> {
    last_change: &'a str,
    title: &'a str,
    description: &'a str,
    tags: &'a str,
    entry_url: &'a str,
}
