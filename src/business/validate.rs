use business::error::ParameterError;
use fast_chemail::is_valid_email;
use url::Url;
use entities::*;
use regex::Regex;

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-z0-9]{1,30}$").unwrap();
}

pub trait Validate {
    fn validate(&self) -> Result<(), ParameterError>;
}

pub fn email(email: &str) -> Result<(), ParameterError> {
    if !is_valid_email(email) {
        return Err(ParameterError::Email);
    }
    Ok(())
}

fn homepage(url: &str) -> Result<(), ParameterError> {
    Url::parse(url).map_err(|_| ParameterError::Url).map(|_| ())
}

fn license(s: &str) -> Result<(), ParameterError> {
    match s {
        "CC0-1.0" | "ODbL-1.0" => Ok(()),
        _ => Err(ParameterError::License),
    }
}

pub fn bbox(bbox: &Bbox) -> Result<(), ParameterError> {
    let lats = vec![bbox.north_east.lat, bbox.south_west.lat];
    let lngs = vec![bbox.north_east.lng, bbox.south_west.lng];
    for lat in lats {
        if lat < -90.0 || lat > 90.0 {
            return Err(ParameterError::Bbox);
        }
    }
    for lng in lngs {
        if lng < -180.0 || lng > 180.0 {
            return Err(ParameterError::Bbox);
        }
    }
    if bbox.north_east.lat == bbox.south_west.lat && bbox.north_east.lng == bbox.south_west.lng {
        return Err(ParameterError::Bbox);
    }
    Ok(())
}

pub fn username(name: &str) -> Result<(), ParameterError> {
    if !USERNAME_REGEX.is_match(name) {
        return Err(ParameterError::UserName);
    }
    Ok(())
}

pub fn password(pw: &str) -> Result<(), ParameterError> {
    //TODO: use regex
    if pw == "" || pw.contains(' ') {
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
    assert!(email("foo@bar").is_err());
    assert!(email("foo@bar.tld").is_ok());
}

#[test]
fn homepage_test() {
    assert!(homepage("https://openfairdb.org").is_ok());
    assert!(homepage("openfairdb.org/foo").is_err());
}

#[test]
fn bbox_test() {
    let c1 = Coordinate {
        lat: 49.123,
        lng: 10.123,
    };
    let c2 = Coordinate {
        lat: 48.123,
        lng: 5.123,
    };
    let c3 = Coordinate {
        lat: 48.123,
        lng: 500.123,
    };
    let valid_bbox = Bbox {
        north_east: c1.clone(),
        south_west: c2.clone(),
    };
    let empty_bbox = Bbox {
        north_east: c1.clone(),
        south_west: c1.clone(),
    };
    let too_large_bbox = Bbox {
        north_east: c1.clone(),
        south_west: c3.clone(),
    };
    assert!(bbox(&valid_bbox).is_ok());
    assert!(bbox(&empty_bbox).is_err());
    assert!(bbox(&too_large_bbox).is_err());
}
