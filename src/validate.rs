// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use error::ValidationError;
use json::Entry;

pub trait Validate {
  fn validate(&self) -> Result<(),ValidationError>;
}

pub fn license(s: &str) -> Result<(),ValidationError> {
  match s {
    "CC0-1.0" |
    "ODbL-1.0" => Ok(()),
    _          => Err(ValidationError::License)
  }
}

impl Validate for Entry {

  fn validate(&self) -> Result<(),ValidationError> {
    self.license
      .clone()
      .ok_or(ValidationError::License)
      .and_then(|ref l|license(l))
  }
}

#[test]
fn license_test(){
  assert!(license("CC0-1.0").is_ok());
  assert!(license("CC0").is_err());
  assert!(license("ODbL-1.0").is_ok());
}
