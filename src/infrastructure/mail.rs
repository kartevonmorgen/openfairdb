use chrono::*;
use fast_chemail::is_valid_email;
use quoted_printable;
use std::io::{Error, ErrorKind, Result};

const FROM_ADDRESS: &str = "\"Karte von morgen\" <no-reply@kartevonmorgen.org>";

pub fn create(to: &[String], subject: &str, body: &str) -> Result<String> {
    let to: Vec<_> = to
        .into_iter()
        .filter(|m| is_valid_email(m))
        .cloned()
        .collect();

    if to.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            "No valid email adresses specified",
        ));
    }

    debug_assert!(!subject.is_empty());
    let subject = format!(
        "=?UTF-8?Q?{}?=",
        quoted_printable::encode_to_str(subject.as_bytes())
    );

    let now = Local::now();

    let email = format!(
        "Date:{date}\r\n\
         From:{from}\r\n\
         To:{to}\r\n\
         Subject:{subject}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n\
         {body}",
        date = now.to_rfc2822(),
        from = FROM_ADDRESS,
        to = to.join(","),
        subject = subject,
        body = body
    );

    debug!("composed email: {}", &email);

    Ok(email)
}

#[cfg(all(not(test), feature = "email"))]
pub mod sendmail {
    use super::*;
    use std::io::prelude::*;
    use std::process::{Command, Stdio};

    pub fn send(mail: &str) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_simple_mail() {
        let mail = create(&vec!["mail@test.org".into()], "My Subject", "Hello Mail").unwrap();
        let expected = "From:\"Karte von morgen\" <no-reply@kartevonmorgen.org>\r\n\
                        To:mail@test.org\r\n\
                        Subject:=?UTF-8?Q?My Subject?=\r\n\
                        MIME-Version: 1.0\r\n\
                        Content-Type: text/plain; charset=utf-8\r\n\r\n\
                        Hello Mail";
        assert!(mail.contains(expected));
    }

    #[test]
    fn check_addresses() {
        assert!(create(&vec![], "foo", "bar").is_err());
        assert!(create(&vec!["not-valid".into()], "foo", "bar").is_err());
    }
}
