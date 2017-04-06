use business::error::ParameterError;
use std::str::FromStr;
use emailaddress::EmailAddress;
use url::Url;
use entities::*;
use regex::Regex;

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-z0-9]{1,12}$").unwrap();
}

pub trait Validate {
    fn validate(&self) -> Result<(), ParameterError>;
}

pub fn email(email: &str) -> Result<(), ParameterError> {
    EmailAddress::from_str(email)
        .map_err(|_| ParameterError::Email)
        .map(|_| ())
}

fn homepage(url: &str) -> Result<(), ParameterError> {
    Url::parse(url)
        .map_err(|_| ParameterError::Url)
        .map(|_| ())
}

fn license(s: &str) -> Result<(), ParameterError> {
    match s {
        "CC0-1.0" | "ODbL-1.0" => Ok(()),
        _ => Err(ParameterError::License),
    }
}

pub fn username(name: &str) -> Result<(), ParameterError> {
    if !USERNAME_REGEX.is_match(name) {
        return Err(ParameterError::UserName);
    }
    Ok(())
}

pub fn password(pw: &str) -> Result<(), ParameterError> {
    //TODO: use regex
    if pw == "" || pw.contains(" ") {
        return Err(ParameterError::Password);
    }
    Ok(())
}

impl Validate for Entry {
    fn validate(&self) -> Result<(), ParameterError> {

        self.license
            .clone()
            .ok_or(ParameterError::License)
            .and_then(|ref l| license(l))?;

        if let Some(ref e) = self.email {
            email(e)?;
        }

        if let Some(ref h) = self.homepage {
            homepage(h)?;
        }

        Ok(())
    }
}

#[test]
fn license_test() {
    assert!(license("CC0-1.0").is_ok());
    assert!(license("CC0").is_err());
    assert!(license("ODbL-1.0").is_ok());
}

#[test]
fn email_test() {
    assert!(email("foo").is_err());
    assert!(email("foo@bar").is_ok());
}

#[test]
fn homepage_test() {
    assert!(homepage("https://openfairdb.org").is_ok());
    assert!(homepage("openfairdb.org/foo").is_err());
}
