#[cfg(not(test))]
use std::{
    io::prelude::*,
    process::{Command, Stdio},
};
use std::{
    io::{Error, ErrorKind, Result},
    thread,
};

use fast_chemail::is_valid_email;
use ofdb_entities::email::*;
use time::format_description::well_known::Rfc2822;

use super::EmailGateway;

#[derive(Debug, Clone)]
pub struct Sendmail {
    from: EmailAddress,
}

impl Sendmail {
    pub fn new(from: EmailAddress) -> Self {
        Self { from }
    }
    fn send(&self, mail: String) {
        // TODO: use tokio::task::spawn_blocking
        thread::spawn(move || {
            if let Err(err) = send_raw(&mail) {
                log::warn!("Could not send e-mail: {}", err);
            }
        });
    }
}

#[cfg(not(test))]
fn send_raw(mail: &str) -> Result<()> {
    let mut child = Command::new("sendmail")
        .arg("-t")
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not get stdin"))?
        .write_all(mail.as_bytes())?;
    child.wait_with_output()?;
    Ok(())
}

/// Don't actually send emails while running the tests or
/// if the `email` feature is disabled.
#[cfg(test)]
fn send_raw(email: &str) -> Result<()> {
    log::debug!("Would send e-mail: {}", email);
    Ok(())
}

impl EmailGateway for Sendmail {
    fn compose_and_send(&self, recipients: &[EmailAddress], content: &EmailContent) {
        log::debug!("Sending e-mails to: {:?}", recipients);
        for to in recipients {
            match compose(&self.from, &[to], content) {
                Ok(email) => {
                    self.send(email);
                }
                Err(err) => {
                    log::warn!("Failed to compose e-mail: {}", err);
                }
            }
        }
    }
}

// quoted_printable limits the length of lines to 76 chars
// and otherwise inserts unintended line breaks! The max.
// length of a header line is 78 chars including the \r\n
// line break.
const MAX_HEADER_FIELD_LEN: usize = 76;

const LINE_BREAK: &str = "\r\n";

fn encode_header_field_partially(input: &str, encoded_max_len: usize) -> (String, usize) {
    // overhead of the encoding (see string formatting literal below)
    debug_assert!(encoded_max_len >= "=?UTF-8?Q??=".len());
    debug_assert!(encoded_max_len <= MAX_HEADER_FIELD_LEN);
    // Try to encode the whole string first, then continue with
    // binary search to find the maximum input length.
    let mut input_min_len = 0;
    let mut input_max_len = input.len() * 2;
    loop {
        debug_assert!(input_min_len <= input_max_len);
        debug_assert!(input.is_char_boundary(input_min_len));
        debug_assert!(input_max_len >= input.len() || input.is_char_boundary(input_max_len));
        let mut input_len = input_min_len + (input_max_len - input_min_len) / 2;
        while !input.is_char_boundary(input_len) {
            input_len -= 1;
        }
        let encoded = format!(
            "=?UTF-8?Q?{}?=",
            quoted_printable::encode_to_str(input[..input_len].as_bytes())
        );
        if encoded.len() <= encoded_max_len {
            if input_len == input_min_len {
                return (encoded, input_len);
            } else {
                // adjust lower bound and continue with binary search
                input_min_len = input_len;
            }
        } else {
            debug_assert!(input_min_len < input_len);
            // adjust upper bound and continue with binary search
            input_max_len = input_len;
        }
    }
}

fn encode_header_field(name: &str, input: &str) -> String {
    let mut prefix_len = name.len() + 1;
    let mut encoded_output = String::with_capacity(prefix_len + input.len() * 2);
    encoded_output.push_str(name);
    encoded_output.push(':');
    let mut input_len = 0;
    while input_len < input.len() {
        if input_len > 0 {
            // append line break and continuation
            encoded_output.push_str(LINE_BREAK);
            encoded_output.push(' ');
            prefix_len = 1;
        }
        let (encoded_part, input_part_len) =
            encode_header_field_partially(&input[input_len..], MAX_HEADER_FIELD_LEN - prefix_len);
        debug_assert!(!encoded_part.is_empty());
        debug_assert!(input_part_len > 0);
        encoded_output.push_str(&encoded_part);
        input_len += input_part_len;
    }
    encoded_output
}

pub fn compose(
    from: &EmailAddress,
    to: &[&EmailAddress],
    email_content: &EmailContent,
) -> Result<String> {
    let to: Vec<_> = to
        .iter()
        .filter(|m| is_valid_email(m.as_str()))
        .map(|m| m.as_str())
        .collect();

    if to.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            "No valid email addresses specified",
        ));
    }

    let EmailContent { subject, body } = email_content;

    let date = time::OffsetDateTime::now_local()
        .ok()
        .and_then(|now| now.format(&Rfc2822).ok())
        .map(|date| format!("Date:{date}\r\n"))
        .unwrap_or_default();

    let to = to.join(",");
    let subject_header = encode_header_field("Subject", subject);

    let email = format!(
        "{date}
         From:{from}\r\n\
         To:{to}\r\n\
         {subject_header}\r\n\
         MIME-Version:1.0\r\n\
         Content-Type:text/plain;charset=utf-8\r\n\r\n\
         {body}",
    );

    log::debug!("composed email: {}", &email);

    Ok(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_simple_mail() {
        let content = EmailContent {
          subject: "My veeeeerrrrryyyyy looooonnnnnggggg Subject with äöüÄÖÜß Umlaute and even more characters that are distributed onto multiple lines".to_string(),
          body: "Hello Mail".to_string(),
        };
        let mail = compose(
            &"\"OFDB\" <from@ofdb.io>".parse::<EmailAddress>().unwrap(),
            &[&"mail@test.org".parse::<EmailAddress>().unwrap()],
            &content,
        )
        .unwrap();
        let expected = "From:\"OFDB\" <from@ofdb.io>\r\n\
             To:mail@test.org\r\n\
             Subject:=?UTF-8?Q?My veeeeerrrrryyyyy looooonnnnnggggg Subject with =C3=A4?=\r\n \
             =?UTF-8?Q?=C3=B6=C3=BC=C3=84=C3=96=C3=9C=C3=9F Umlaute and even more char?=\r\n \
             =?UTF-8?Q?acters that are distributed onto multiple lines?=\r\n\
             MIME-Version:1.0\r\n\
             Content-Type:text/plain;charset=utf-8\r\n\r\n\
             Hello Mail";
        assert!(mail.contains(expected));
    }

    #[test]
    fn check_addresses() {
        let content = EmailContent {
            subject: "foo".to_string(),
            body: "bar".to_string(),
        };
        assert!(compose(
            &"from@mail.org".parse::<EmailAddress>().unwrap(),
            &[],
            &content
        )
        .is_err());
        assert!(compose(
            &EmailAddress::new_unchecked("from".to_string()),
            &[&EmailAddress::new_unchecked("not-valid".to_string())],
            &content
        )
        .is_err());
    }
}
