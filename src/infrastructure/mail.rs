use std::process::{Command, Stdio};
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use chrono::*;
use quoted_printable::encode;
use fast_chemail::is_valid_email;

const FROM_ADDRESS: &str = "\"Karte von morgen\" <no-reply@kartevonmorgen.org>";

pub fn create(to: &[String], subject: &str, body: &str) -> Result<String> {
    let to: Vec<_> = to.into_iter()
        .filter(|m| is_valid_email(m))
        .cloned()
        .collect();

    if to.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            "No valid email adresses specified",
        ));
    }

    let now = Local::now().format("%d %b %Y %H:%M:%S %z").to_string();

    let subject = format!(
        "=?UTF-8?Q?{}?=",
        String::from_utf8_lossy(&encode(subject.as_bytes()))
    );

    let email = format!(
        "Date:{date}\r\n\
         From:{from}\r\n\
         To:{to}\r\n\
         Subject:{subject}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n\
         {body}",
        date = now.as_str(),
        from = FROM_ADDRESS,
        to = to.join(","),
        subject = subject,
        body = body
    );

    debug!("sending email: {}", &email);

    Ok(email)
}

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
