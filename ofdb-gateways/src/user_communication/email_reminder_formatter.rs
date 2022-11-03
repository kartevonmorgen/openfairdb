use askama::Template;
use ofdb_core::usecases::{EmailReminderFormatter, Reminder};

#[derive(Default)]
pub struct ReminderFormatter; // TODO: support different languages

impl EmailReminderFormatter for ReminderFormatter {
    fn subject(&self, r: &Reminder) -> String {
        let tpl = EmailReminderScoutsSubjectTemplate {
            title: &r.place.title,
        };
        tpl.render().unwrap()
    }
    fn body(&self, r: &Reminder) -> String {
        let tpl = EmailReminderScoutsBodyTemplate {
            title: &r.place.title,
            description: &r.place.description,
            id: r.place.id.as_str(),
        };
        tpl.render().unwrap()
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
