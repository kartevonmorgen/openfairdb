// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use error::ValidationError;
use json::Entry;
use emailaddress::EmailAddress;
use std::str::FromStr;

pub trait Validate {
  fn validate(&self) -> Result<(),ValidationError>;
}

fn email(s: &str) -> Result<(),ValidationError> {
  EmailAddress::from_str(s)
    .map_err(|_|ValidationError::Email)
    .map(|_|())
}

fn license(s: &str) -> Result<(),ValidationError> {
  match s {
    "CC0-1.0" |
    "ODbL-1.0" => Ok(()),
    _          => Err(ValidationError::License)
  }
}

impl Validate for Entry {

  fn validate(&self) -> Result<(),ValidationError> {

    try!(self.license
      .clone()
      .ok_or(ValidationError::License)
      .and_then(|ref l|license(l)));

    if let Some(ref e) = self.email.clone() {
      try!(email(e));
    }

    Ok(())
  }
}

#[test]
fn license_test(){
  assert!(license("CC0-1.0").is_ok());
  assert!(license("CC0").is_err());
  assert!(license("ODbL-1.0").is_ok());
}

#[test]
fn email_test(){
  assert!(email("foo").is_err());
  assert!(email("foo@bar").is_ok());
}
