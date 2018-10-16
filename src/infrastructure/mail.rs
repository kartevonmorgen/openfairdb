use failure::Fallible;
use fast_chemail::is_valid_email;
use lettre::EmailTransport;
use lettre_email::{Email, EmailBuilder};

#[cfg(all(not(test)))]
use lettre::sendmail::SendmailTransport;

#[cfg(all(test))]
use lettre::stub::StubEmailTransport;

const FROM_ADDRESS: &str = "\"Karte von morgen\" <no-reply@kartevonmorgen.org>";

const SENDMAIL_COMMAND: &str = "sendmail";

pub fn create_email(addresses: &[String], subject: &str, text: &str) -> Fallible<Email> {
    let addresses: Vec<_> = addresses
        .into_iter()
        .filter(|m| is_valid_email(m))
        .cloned()
        .collect();
    if addresses.is_empty() {
        bail!("No valid email addresses specified");
    }

    let email = addresses
        .into_iter()
        .fold(EmailBuilder::new(), |mut builder, address| {
            builder.add_to(address);
            builder
        }).from(FROM_ADDRESS)
        .subject(subject)
        .text(text)
        .build()?;

    Ok(email)
}

#[cfg(not(test))]
pub fn send_email(email: &Email) -> Fallible<()> {
    SendmailTransport::new_with_command(SENDMAIL_COMMAND)
        .send(email)
        .map_err(Into::into)
}

#[cfg(test)]
pub fn send_email(email: &Email) -> Fallible<()> {
    let _ = StubEmailTransport::new_positive().send(email);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lettre::SendableEmail;

    #[test]
    fn create_simple_mail() {
        let subject = "My verrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrry loooooooooooooooooooooong Subject containing special characters ÄÖÜäöüßéàí";
        let email = create_email(&vec!["mail@test.org".into()], subject, "Hello Mail").unwrap();
        let message = String::from_utf8_lossy(&email.message());
        assert!(message.contains("From: <\"Karte von morgen\" <no-reply@kartevonmorgen.org>>\r\n"));
        assert!(message.contains("To: <mail@test.org>\r\n"));
        assert!(message.contains("Hello Mail\r\n"));
        // subject, 1st line
        assert!(
            message.contains("Subject: My verrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrry\r\n")
        );
        // subject, 2nd line
        assert!(
            message.contains("loooooooooooooooooooooong Subject containing special characters ÄÖÜäöüßéàí\r\n")
        );
    }

    #[test]
    fn check_addresses() {
        assert!(create_email(&vec![], "foo", "bar").is_err());
        assert!(create_email(&vec!["not-valid".into()], "foo", "bar").is_err());
    }
}
