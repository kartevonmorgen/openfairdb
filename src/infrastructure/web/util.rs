use regex::Regex;
use entities::*;
use adapters::user_communication;
use business::usecase;
use super::mail;

lazy_static! {
    static ref HASH_TAG_REGEX: Regex = Regex::new(r"#(?P<tag>\w+((-\w+)*)?)").unwrap();
}

pub fn extract_ids(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.to_owned())
        .filter(|id| id != "")
        .collect::<Vec<String>>()
}

#[cfg(feature = "email")]
pub fn send_mails(email_addresses: &[String], subject: &str, body: &str) {
    debug!("sending emails to: {:?}", email_addresses);
    for email_address in email_addresses.to_owned() {
        let to = vec![email_address];
        match mail::create(&to, subject, body) {
            Ok(mail) => {
                ::std::thread::spawn(move || {
                    if let Err(err) = mail::send(&mail) {
                        warn!("Could not send mail: {}", err);
                    }
                });
            }
            Err(e) => {
                warn!("could not create notification mail: {}", e);
            }
        }
    }
}

#[cfg(not(feature = "email"))]
pub fn send_mails(_: Vec<String>, _: &str, _: &str) {
    // do nothing
}

pub fn notify_create_entry(
    email_addresses: &[String],
    e: &usecase::NewEntry,
    id: &str,
    all_categories: Vec<Category>,
) {
    let subject = String::from("Karte von Morgen - neuer Eintrag: ") + &e.title;
    let categories: Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::new_entry_email(e, id, &categories);
    send_mails(email_addresses, &subject, &body);
}

pub fn notify_update_entry(
    email_addresses: &[String],
    e: &usecase::UpdateEntry,
    all_categories: Vec<Category>,
) {
    let subject = String::from("Karte von Morgen - Eintrag verändert: ") + &e.title;
    let categories: Vec<String> = all_categories
        .into_iter()
        .filter(|c| e.categories.clone().into_iter().any(|c_id| *c.id == c_id))
        .map(|c| c.name)
        .collect();
    let body = user_communication::changed_entry_email(e, &categories);
    send_mails(email_addresses, &subject, &body);
}

pub fn extract_hash_tags(text: &str) -> Vec<String> {
    let mut res: Vec<String> = vec![];
    for cap in HASH_TAG_REGEX.captures_iter(text) {
        res.push(cap["tag"].into());
    }
    res
}

pub fn remove_hash_tags(text: &str) -> String {
    HASH_TAG_REGEX
        .replace_all(text, "")
        .into_owned()
        .replace("  ", " ")
        .trim()
        .into()
}
