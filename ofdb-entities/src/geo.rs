use std::{
    num::ParseFloatError,
    ops::{Add, Sub},
};

use itertools::*;
use thiserror::Error;

pub type RawCoord = i32;

// Assumption: 2-complement binary representation
const RAW_COORD_INVALID: RawCoord = std::i32::MIN;
const RAW_COORD_MAX: RawCoord = std::i32::MAX;
const RAW_COORD_MIN: RawCoord = -RAW_COORD_MAX;

#[derive(Debug, Error)]
pub enum CoordRangeError {
    #[error("degrees out of range")]
    Degrees(f64),

    #[error("radians out of range")]
    Radians(f64),
}

#[derive(Debug, Error)]
pub enum CoordInputError {
    #[error(transparent)]
    Range(#[from] CoordRangeError),

    #[error(transparent)]
    Parse(#[from] ParseFloatError),
}

/// Compact fixed-point integer representation of a geographical coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeoCoord(RawCoord);

impl GeoCoord {
    const INVALID: Self = Self(RAW_COORD_INVALID);

    pub const fn max() -> Self {
        Self(RAW_COORD_MAX)
    }

    pub const fn min() -> Self {
        Self(RAW_COORD_MIN)
    }

    pub const fn to_raw(self) -> RawCoord {
        self.0
    }

    pub const fn from_raw(raw: RawCoord) -> Self {
        Self(raw)
    }

    pub fn is_valid(self) -> bool {
        self != Self::INVALID
    }
}

impl Default for GeoCoord {
    fn default() -> Self {
        let res = Self::INVALID;
        debug_assert!(!res.is_valid());
        res
    }
}

impl std::cmp::PartialOrd for GeoCoord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(std::cmp::Ordering::Equal)
        } else if self.is_valid() && other.is_valid() {
            Some(self.to_raw().cmp(&other.to_raw()))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd)]
pub struct LatCoord(GeoCoord);

impl LatCoord {
    const RAD_MAX: f64 = std::f64::consts::FRAC_PI_2;
    const RAD_MIN: f64 = -std::f64::consts::FRAC_PI_2;
    const TO_RAD: f64 =
        (Self::RAD_MAX - Self::RAD_MIN) / (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64);
    const FROM_RAD: f64 =
        (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64) / (Self::RAD_MAX - Self::RAD_MIN);

    const DEG_MAX: f64 = 90.0;
    const DEG_MIN: f64 = -90.0;
    const TO_DEG: f64 =
        (Self::DEG_MAX - Self::DEG_MIN) / (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64);
    const FROM_DEG: f64 =
        (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64) / (Self::DEG_MAX - Self::DEG_MIN);

    pub const fn max() -> Self {
        Self(GeoCoord::max())
    }

    pub const fn min() -> Self {
        Self(GeoCoord::min())
    }

    pub const fn to_raw(self) -> RawCoord {
        self.0.to_raw()
    }

    pub const fn from_raw(raw: RawCoord) -> Self {
        Self(GeoCoord::from_raw(raw))
    }

    pub fn is_valid(self) -> bool {
        self.0.is_valid()
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn to_rad(self) -> f64 {
        if self.is_valid() {
            debug_assert!(self.to_raw() >= RAW_COORD_MIN);
            debug_assert!(self.to_raw() <= RAW_COORD_MAX);
            let rad = f64::from(self.to_raw()) * Self::TO_RAD;
            debug_assert!(rad >= Self::RAD_MIN);
            debug_assert!(rad <= Self::RAD_MAX);
            rad
        } else {
            std::f64::NAN
        }
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn to_deg(self) -> f64 {
        if self.is_valid() {
            debug_assert!(self.to_raw() >= RAW_COORD_MIN);
            debug_assert!(self.to_raw() <= RAW_COORD_MAX);
            let deg = f64::from(self.to_raw()) * Self::TO_DEG;
            debug_assert!(deg >= Self::DEG_MIN);
            debug_assert!(deg <= Self::DEG_MAX);
            deg
        } else {
            std::f64::NAN
        }
    }

    pub fn from_rad<T: Into<f64>>(rad: T) -> Self {
        let rad = rad.into();
        debug_assert!(rad >= Self::RAD_MIN);
        debug_assert!(rad <= Self::RAD_MAX);
        let raw = f64::round(rad * Self::FROM_RAD) as RawCoord;
        let res = Self::from_raw(raw);
        debug_assert!(res.is_valid());
        res
    }

    pub fn from_deg<T: Into<f64>>(deg: T) -> Self {
        let deg = deg.into();
        debug_assert!(deg >= Self::DEG_MIN);
        debug_assert!(deg <= Self::DEG_MAX);
        let raw = f64::round(deg * Self::FROM_DEG) as RawCoord;
        let res = Self::from_raw(raw);
        debug_assert!(res.is_valid());
        res
    }

    pub fn try_from_deg<T: Into<f64>>(deg: T) -> Result<Self, CoordRangeError> {
        let deg = deg.into();
        if (Self::DEG_MIN..=Self::DEG_MAX).contains(&deg) {
            Ok(Self::from_deg(deg))
        } else {
            Err(CoordRangeError::Degrees(deg))
        }
    }
}

impl Add for LatCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut deg = self.to_deg() + rhs.to_deg();
        if deg > Self::DEG_MAX {
            deg -= Self::DEG_MAX - Self::DEG_MIN;
        }
        Self::from_deg(deg)
    }
}

impl Sub for LatCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut deg = self.to_deg() - rhs.to_deg();
        if deg < Self::DEG_MIN {
            deg += Self::DEG_MAX - Self::DEG_MIN;
        }
        Self::from_deg(deg)
    }
}

impl std::fmt::Display for LatCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_deg())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd)]
pub struct LngCoord(GeoCoord);

impl LngCoord {
    const RAD_MAX: f64 = std::f64::consts::PI;
    const RAD_MIN: f64 = -std::f64::consts::PI;
    const TO_RAD: f64 =
        (Self::RAD_MAX - Self::RAD_MIN) / (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64);
    const FROM_RAD: f64 =
        (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64) / (Self::RAD_MAX - Self::RAD_MIN);

    const DEG_MAX: f64 = 180.0;
    const DEG_MIN: f64 = -180.0;
    const TO_DEG: f64 =
        (Self::DEG_MAX - Self::DEG_MIN) / (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64);
    const FROM_DEG: f64 =
        (RAW_COORD_MAX as f64 - RAW_COORD_MIN as f64) / (Self::DEG_MAX - Self::DEG_MIN);

    pub const fn max() -> Self {
        Self(GeoCoord::max())
    }

    pub const fn min() -> Self {
        Self(GeoCoord::min())
    }

    pub const fn to_raw(self) -> RawCoord {
        self.0.to_raw()
    }

    pub const fn from_raw(raw: RawCoord) -> Self {
        Self(GeoCoord::from_raw(raw))
    }

    pub fn is_valid(self) -> bool {
        self.0.is_valid()
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn to_rad(self) -> f64 {
        if self.is_valid() {
            debug_assert!(self.to_raw() >= RAW_COORD_MIN);
            debug_assert!(self.to_raw() <= RAW_COORD_MAX);
            let rad = f64::from(self.to_raw()) * Self::TO_RAD;
            debug_assert!(rad >= Self::RAD_MIN);
            debug_assert!(rad <= Self::RAD_MAX);
            rad
        } else {
            std::f64::NAN
        }
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    pub fn to_deg(self) -> f64 {
        if self.is_valid() {
            debug_assert!(self.to_raw() >= RAW_COORD_MIN);
            debug_assert!(self.to_raw() <= RAW_COORD_MAX);
            let deg = f64::from(self.to_raw()) * Self::TO_DEG;
            debug_assert!(deg >= Self::DEG_MIN);
            debug_assert!(deg <= Self::DEG_MAX);
            deg
        } else {
            std::f64::NAN
        }
    }

    pub fn from_rad<T: Into<f64>>(rad: T) -> Self {
        let rad = rad.into();
        debug_assert!(rad >= Self::RAD_MIN);
        debug_assert!(rad <= Self::RAD_MAX);
        let raw = f64::round(rad * Self::FROM_RAD) as RawCoord;
        let res = Self::from_raw(raw);
        debug_assert!(res.is_valid());
        res
    }

    pub fn from_deg<T: Into<f64>>(deg: T) -> Self {
        let deg = deg.into();
        debug_assert!(deg >= Self::DEG_MIN);
        debug_assert!(deg <= Self::DEG_MAX);
        let raw = f64::round(deg * Self::FROM_DEG) as RawCoord;
        let res = Self::from_raw(raw);
        debug_assert!(res.is_valid());
        res
    }

    pub fn try_from_deg<T: Into<f64>>(deg: T) -> Result<Self, CoordRangeError> {
        let deg = deg.into();
        if (Self::DEG_MIN..=Self::DEG_MAX).contains(&deg) {
            Ok(Self::from_deg(deg))
        } else {
            Err(CoordRangeError::Degrees(deg))
        }
    }
}

impl Add for LngCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut deg = self.to_deg() + rhs.to_deg();
        if deg > Self::DEG_MAX {
            deg -= Self::DEG_MAX - Self::DEG_MIN;
        }
        Self::from_deg(deg)
    }
}

impl Sub for LngCoord {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut deg = self.to_deg() - rhs.to_deg();
        if deg < Self::DEG_MIN {
            deg += Self::DEG_MAX - Self::DEG_MIN;
        }
        Self::from_deg(deg)
    }
}

impl std::fmt::Display for LngCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_deg())
    }
}

/// Compact internal representation of a geographical location on a (flat) map.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapPoint {
    lat: LatCoord,
    lng: LngCoord,
}

impl MapPoint {
    pub const fn new(lat: LatCoord, lng: LngCoord) -> Self {
        Self { lat, lng }
    }

    pub const fn lat(self) -> LatCoord {
        self.lat
    }

    pub const fn lng(self) -> LngCoord {
        self.lng
    }

    pub fn is_valid(self) -> bool {
        self.lat.is_valid() && self.lng.is_valid()
    }

    pub fn to_lat_lng_rad(self) -> (f64, f64) {
        (self.lat.to_rad(), self.lng.to_rad())
    }

    pub fn to_lat_lng_deg(self) -> (f64, f64) {
        (self.lat.to_deg(), self.lng.to_deg())
    }

    pub fn from_lat_lng_deg<LAT: Into<f64>, LNG: Into<f64>>(lat: LAT, lng: LNG) -> Self {
        Self::new(LatCoord::from_deg(lat), LngCoord::from_deg(lng))
    }

    pub fn try_from_lat_lng_deg<LAT: Into<f64>, LNG: Into<f64>>(
        lat: LAT,
        lng: LNG,
    ) -> Result<Self, CoordRangeError> {
        let lat = LatCoord::try_from_deg(lat)?;
        let lng = LngCoord::try_from_deg(lng)?;
        Ok(Self::new(lat, lng))
    }

    fn parse_lat_lng_deg(lat_deg_str: &str, lng_deg_str: &str) -> Result<Self, MapPointInputError> {
        let lat_deg = lat_deg_str
            .parse::<f64>()
            .map_err(|err| MapPointInputError::Latitude(err.into()))?;
        let lat = LatCoord::try_from_deg(lat_deg)
            .map_err(|err| MapPointInputError::Latitude(err.into()))?;
        debug_assert!(lat.is_valid());
        let lng_deg = lng_deg_str
            .parse::<f64>()
            .map_err(|err| MapPointInputError::Longitude(err.into()))?;
        let lng = LngCoord::try_from_deg(lng_deg)
            .map_err(|err| MapPointInputError::Longitude(err.into()))?;
        debug_assert!(lng.is_valid());
        Ok(MapPoint::new(lat, lng))
    }
}

#[derive(Debug, Error)]
pub enum MapPointInputError {
    #[error("latitude: {0}")]
    Latitude(CoordInputError),

    #[error("longitude: {0}")]
    Longitude(CoordInputError),

    #[error("invalid format: '{0}'")]
    Format(String),
}

impl std::fmt::Display for MapPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{},{}", self.lat, self.lng)
    }
}

impl std::str::FromStr for MapPoint {
    type Err = MapPointInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((lat_deg_str, lng_deg_str)) = s.split(',').collect_tuple() {
            MapPoint::parse_lat_lng_deg(lat_deg_str, lng_deg_str)
        } else {
            Err(MapPointInputError::Format(s.to_string()))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Distance(pub f64);

impl Distance {
    pub const fn infinite() -> Self {
        Self(std::f64::INFINITY)
    }

    pub const fn from_meters(meters: f64) -> Self {
        Self(meters)
    }

    pub const fn to_meters(self) -> f64 {
        self.0
    }

    pub fn is_valid(self) -> bool {
        self.0 >= 0.0
    }
}

// Semi-axes of WGS-84 ellipsoid
const WGS84_MAJOR_SEMIAXIS: Distance = Distance::from_meters(6_378_137.0);
const WGS84_MINOR_SEMIAXIS: Distance = Distance::from_meters(6_356_752.3);

// Earth radius at a given latitude, according to the WGS-84 ellipsoid
fn wgs84_earth_radius(lat: LatCoord) -> Distance {
    let major_n =
        WGS84_MAJOR_SEMIAXIS.to_meters() * WGS84_MAJOR_SEMIAXIS.to_meters() * lat.to_rad().cos();
    let minor_n =
        WGS84_MINOR_SEMIAXIS.to_meters() * WGS84_MINOR_SEMIAXIS.to_meters() * lat.to_rad().sin();
    let major_d = WGS84_MAJOR_SEMIAXIS.to_meters() * lat.to_rad().cos();
    let minor_d = WGS84_MINOR_SEMIAXIS.to_meters() * lat.to_rad().sin();
    Distance::from_meters(
        ((major_n * major_n + minor_n * minor_n) / (major_d * major_d + minor_d * minor_d)).sqrt(),
    )
}

impl MapPoint {
    /// Calculate the great-circle distance on the surface
    /// of the earth using a special case of the Vincenty
    /// formula for numerical accuracy.
    /// Reference: https://en.wikipedia.org/wiki/Great-circle_distance
    pub fn distance(p1: MapPoint, p2: MapPoint) -> Option<Distance> {
        if !p1.is_valid() || !p2.is_valid() {
            return None;
        }

        let (lat1_rad, lng1_rad) = p1.to_lat_lng_rad();
        let (lat2_rad, lng2_rad) = p2.to_lat_lng_rad();

        let (lat1_sin, lat1_cos) = (lat1_rad.sin(), lat1_rad.cos());
        let (lat2_sin, lat2_cos) = (lat2_rad.sin(), lat2_rad.cos());

        let dlng = (lng1_rad - lng2_rad).abs();
        let (dlng_sin, dlng_cos) = (dlng.sin(), dlng.cos());

        let nom1 = lat2_cos * dlng_sin;
        let nom2 = lat1_cos * lat2_sin - lat1_sin * lat2_cos * dlng_cos;

        let nom = (nom1 * nom1 + nom2 * nom2).sqrt();
        let denom = lat1_sin * lat2_sin + lat1_cos * lat2_cos * dlng_cos;

        let mean_earth_radius_meters = (wgs84_earth_radius(p1.lat()).to_meters()
            + wgs84_earth_radius(p2.lat()).to_meters())
            / 2.0;
        Some(Distance::from_meters(
            mean_earth_radius_meters * nom.atan2(denom),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MapBbox {
    sw: MapPoint,
    ne: MapPoint,
}

impl MapBbox {
    pub const fn new(sw: MapPoint, ne: MapPoint) -> Self {
        Self { sw, ne }
    }

    /// Create a bounding box with a center point and given width/height
    pub fn centered_around(center: MapPoint, lat_width: Distance, lng_height: Distance) -> Self {
        let lat = center.lat();
        let lng = center.lng();

        // Radius of the parallel at given latitude
        let earth_radius = wgs84_earth_radius(lat).to_meters();
        let parallel_radius = earth_radius * lat.to_rad().cos();

        let lat_delta = LatCoord::from_rad(lat_width.to_meters() / (2.0 * parallel_radius));
        let lng_delta = LngCoord::from_rad(lng_height.to_meters() / (2.0 * parallel_radius));

        let sw = MapPoint::new(lat - lat_delta, lng - lng_delta);
        let ne = MapPoint::new(lat + lat_delta, lng + lng_delta);

        Self::new(sw, ne)
    }

    pub const fn southwest(&self) -> MapPoint {
        self.sw
    }

    pub const fn northeast(&self) -> MapPoint {
        self.ne
    }

    pub fn is_valid(&self) -> bool {
        self.sw.is_valid() && self.ne.is_valid() && self.sw.lat() <= self.ne.lat()
    }

    pub fn is_empty(&self) -> bool {
        debug_assert!(self.sw.is_valid());
        debug_assert!(self.ne.is_valid());
        self.sw.lat() >= self.ne.lat() || self.sw.lng() == self.ne.lng()
    }

    pub fn contains_point(&self, pt: MapPoint) -> bool {
        debug_assert!(self.is_valid());
        debug_assert!(pt.is_valid());
        if pt.lat() < self.sw.lat() || pt.lat() > self.ne.lat() {
            return false;
        }
        if self.sw.lng() <= self.ne.lng() {
            // regular (inclusive)
            pt.lng() >= self.sw.lng() && pt.lng() <= self.ne.lng()
        } else {
            // inverse (exclusive)
            !(pt.lng() > self.ne.lng() && pt.lng() < self.sw.lng())
        }
    }
}

impl std::fmt::Display for MapBbox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{},{}", self.sw, self.ne)
    }
}

impl std::str::FromStr for MapBbox {
    type Err = MapBboxInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((sw_lat_deg_str, sw_lng_deg_str, ne_lat_deg_str, ne_lng_deg_str)) =
            s.split(',').collect_tuple()
        {
            let sw = MapPoint::parse_lat_lng_deg(sw_lat_deg_str, sw_lng_deg_str)
                .map_err(MapBboxInputError::Southwest)?;
            let ne = MapPoint::parse_lat_lng_deg(ne_lat_deg_str, ne_lng_deg_str)
                .map_err(MapBboxInputError::Northeast)?;
            Ok(MapBbox::new(sw, ne))
        } else {
            Err(MapBboxInputError::Format(s.to_string()))
        }
    }
}

#[derive(Debug, Error)]
pub enum MapBboxInputError {
    #[error("southwest point: {0}")]
    Southwest(MapPointInputError),

    #[error("northeast point: {0}")]
    Northeast(MapPointInputError),

    #[error("invalid format: '{0}'")]
    Format(String),
}

#[cfg(test)]
#[allow(clippy::unreadable_literal, clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn latitude() {
        assert!(!LatCoord::default().is_valid());
        assert!(LatCoord::default().to_deg().is_nan());
        assert_eq!(0.0, LatCoord::from_raw(0).to_deg());
        assert_eq!(RAW_COORD_MIN, LatCoord::min().to_raw());
        assert_eq!(RAW_COORD_MAX, LatCoord::max().to_raw());
        assert_eq!(
            LatCoord::min(),
            LatCoord::from_raw(LatCoord::min().to_raw())
        );
        assert_eq!(
            LatCoord::max(),
            LatCoord::from_raw(LatCoord::max().to_raw())
        );
        assert_eq!(
            LatCoord::min(),
            LatCoord::from_deg(LatCoord::min().to_deg())
        );
        assert_eq!(
            LatCoord::max(),
            LatCoord::from_deg(LatCoord::max().to_deg())
        );
        assert_eq!(LatCoord::min(), LatCoord::from_deg(-90));
        assert_eq!(LatCoord::max(), LatCoord::from_deg(90));
        assert!(LatCoord::try_from_deg(-90.000001).is_err());
        assert!(LatCoord::try_from_deg(90.000001).is_err());
    }

    #[test]
    fn longitude() {
        assert!(!LngCoord::default().is_valid());
        assert!(LngCoord::default().to_deg().is_nan());
        assert_eq!(0.0, LngCoord::from_raw(0).to_deg());
        assert_eq!(RAW_COORD_MIN, LngCoord::min().to_raw());
        assert_eq!(RAW_COORD_MAX, LngCoord::max().to_raw());
        assert!(LngCoord::min().is_valid());
        assert!(LngCoord::max().is_valid());
        assert_eq!(
            LngCoord::min(),
            LngCoord::from_raw(LngCoord::min().to_raw())
        );
        assert_eq!(
            LngCoord::max(),
            LngCoord::from_raw(LngCoord::max().to_raw())
        );
        assert_eq!(
            LngCoord::min(),
            LngCoord::from_deg(LngCoord::min().to_deg())
        );
        assert_eq!(
            LngCoord::max(),
            LngCoord::from_deg(LngCoord::max().to_deg())
        );
        assert_eq!(LngCoord::min(), LngCoord::from_deg(-180));
        assert_eq!(LngCoord::max(), LngCoord::from_deg(180));
        assert!(LngCoord::try_from_deg(-180.000001).is_err());
        assert!(LngCoord::try_from_deg(180.000001).is_err());
    }

    #[test]
    fn no_distance() {
        let p1 = MapPoint::from_lat_lng_deg(0.0, 0.0);
        assert_eq!(MapPoint::distance(p1, p1).unwrap().to_meters(), 0.0);

        let p2 = MapPoint::from_lat_lng_deg(-25.0, 55.0);
        assert_eq!(MapPoint::distance(p2, p2).unwrap().to_meters(), 0.0);

        let p1 = MapPoint::from_lat_lng_deg(-15.0, -180.0);
        let p2 = MapPoint::from_lat_lng_deg(-15.0, 180.0);
        assert!(MapPoint::distance(p1, p2).unwrap().to_meters() < 0.000001);
    }

    #[test]
    fn real_distance() {
        let stuttgart = MapPoint::from_lat_lng_deg(48.7755, 9.1827);
        let mannheim = MapPoint::from_lat_lng_deg(49.4836, 8.4630);
        assert!(MapPoint::distance(stuttgart, mannheim).unwrap() > Distance::from_meters(94_000.0));
        assert!(MapPoint::distance(stuttgart, mannheim).unwrap() < Distance::from_meters(95_000.0));

        let new_york = MapPoint::from_lat_lng_deg(40.714268, -74.005974);
        let sidney = MapPoint::from_lat_lng_deg(-33.867138, 151.207108);
        assert!(
            MapPoint::distance(new_york, sidney).unwrap() > Distance::from_meters(15_985_000.0)
        );
        assert!(
            MapPoint::distance(new_york, sidney).unwrap() < Distance::from_meters(15_995_000.0)
        );
    }

    #[test]
    fn symetric_distance() {
        let a = MapPoint::from_lat_lng_deg(80.0, 0.0);
        let b = MapPoint::from_lat_lng_deg(90.0, 20.0);
        assert_eq!(
            MapPoint::distance(a, b).unwrap(),
            MapPoint::distance(b, a).unwrap()
        );
    }

    #[test]
    fn distance_with_invalid_coordinates() {
        let a = MapPoint::new(LatCoord::from_deg(10.0), Default::default());
        let b = MapPoint::from_lat_lng_deg(20.0, 20.0);
        assert_eq!(None, MapPoint::distance(a, b));
    }

    #[test]
    fn positive_distance_regressions() {
        let p1 = MapPoint::from_lat_lng_deg(-81.2281041784343, 77.75747775927069);
        let p2 = MapPoint::from_lat_lng_deg(40.92116510538438, -93.33303223984923);
        assert!(MapPoint::distance(p1, p2).unwrap().to_meters() >= 0.0);

        let p1 = MapPoint::from_lat_lng_deg(67.01568147028595, 122.10276824520099);
        let p2 = MapPoint::from_lat_lng_deg(-87.84709362678561, 132.71691422570353);
        assert!(MapPoint::distance(p1, p2).unwrap().to_meters() >= 0.0);

        let p1 = MapPoint::from_lat_lng_deg(-37.44489137895633, -124.46758920534867);
        let p2 = MapPoint::from_lat_lng_deg(29.29724492099939, 0.03218860366949281);
        assert!(MapPoint::distance(p1, p2).unwrap().to_meters() >= 0.0);
    }

    #[test]
    fn bbox_contains_point() {
        let sw = MapPoint::from_lat_lng_deg(-25.0, -20.0);
        let ne = MapPoint::from_lat_lng_deg(25.0, 30.0);
        let bbox = MapBbox::new(sw, ne);
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, -15.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-26.0, -15.0)));
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, 20.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(26.0, 20.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, -21.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, 31.0)));

        let sw = MapPoint::from_lat_lng_deg(-25.0, 175.0);
        let ne = MapPoint::from_lat_lng_deg(25.0, -175.0);
        let bbox = MapBbox::new(sw, ne);
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, 177.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-26.0, 177.0)));
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, -177.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(26.0, 177.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, 174.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, -174.0)));

        let sw = MapPoint::from_lat_lng_deg(-25.0, 30.0);
        let ne = MapPoint::from_lat_lng_deg(25.0, 10.0);
        let bbox = MapBbox::new(sw, ne);
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, 5.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-26.0, 5.0)));
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, 35.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(26.0, 35.0)));
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, 180.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(26.0, 180.0)));
        assert!(bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, -180.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(26.0, -180.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(-10.0, 11.0)));
        assert!(!bbox.contains_point(MapPoint::from_lat_lng_deg(10.0, 29.0)));

        let bbox1 = MapBbox::new(
            MapPoint::from_lat_lng_deg(0.0, 0.0),
            MapPoint::from_lat_lng_deg(10.0, 10.0),
        );
        let bbox2 = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, 0.0),
            MapPoint::from_lat_lng_deg(0.0, 10.0),
        );
        let bbox3 = MapBbox::new(
            MapPoint::from_lat_lng_deg(-10.0, -10.0),
            MapPoint::from_lat_lng_deg(0.0, 0.0),
        );
        let bbox4 = MapBbox::new(
            MapPoint::from_lat_lng_deg(0.0, -10.0),
            MapPoint::from_lat_lng_deg(10.0, 0.0),
        );

        let lat1 = 5.0;
        let lng1 = 5.0;
        let lat2 = -5.0;
        let lng2 = 5.0;
        let lat3 = -5.0;
        let lng3 = -5.0;
        let lat4 = 5.0;
        let lng4 = -5.0;

        assert!(bbox1.contains_point(MapPoint::from_lat_lng_deg(lat1, lng1)));
        assert!(!bbox2.contains_point(MapPoint::from_lat_lng_deg(lat1, lng1)));
        assert!(!bbox3.contains_point(MapPoint::from_lat_lng_deg(lat1, lng1)));
        assert!(!bbox4.contains_point(MapPoint::from_lat_lng_deg(lat1, lng1)));

        assert!(!bbox1.contains_point(MapPoint::from_lat_lng_deg(lat2, lng2)));
        assert!(bbox2.contains_point(MapPoint::from_lat_lng_deg(lat2, lng2)));
        assert!(!bbox3.contains_point(MapPoint::from_lat_lng_deg(lat2, lng2)));
        assert!(!bbox4.contains_point(MapPoint::from_lat_lng_deg(lat2, lng2)));

        assert!(!bbox1.contains_point(MapPoint::from_lat_lng_deg(lat3, lng3)));
        assert!(!bbox2.contains_point(MapPoint::from_lat_lng_deg(lat3, lng3)));
        assert!(bbox3.contains_point(MapPoint::from_lat_lng_deg(lat3, lng3)));
        assert!(!bbox4.contains_point(MapPoint::from_lat_lng_deg(lat3, lng3)));

        assert!(!bbox1.contains_point(MapPoint::from_lat_lng_deg(lat4, lng4)));
        assert!(!bbox2.contains_point(MapPoint::from_lat_lng_deg(lat4, lng4)));
        assert!(!bbox3.contains_point(MapPoint::from_lat_lng_deg(lat4, lng4)));
        assert!(bbox4.contains_point(MapPoint::from_lat_lng_deg(lat4, lng4)));
    }

    // ---- BENCHMARKS ---- //
    //
    // To run the benchmarks you need Rust nightly.
    // Add the following to lib.rs:
    //
    // ```
    // #![feature(test)]
    //
    // #[cfg(test)]
    // extern crate test;
    // ```
    //
    // Then comment out the following:
    //
    // use crate::test::Bencher;
    //
    // fn random_map_point<T: rand::Rng>(rng: &mut T) -> MapPoint {
    //     let lat = rng.gen_range(LatCoord::min().to_deg(),
    // LatCoord::max().to_deg());     let lng =
    // rng.gen_range(LngCoord::min().to_deg(), LngCoord::max().to_deg());
    //     MapPoint::from_lat_lng_deg(lat, lng)
    // }
    //
    // #[bench]
    // fn bench_distance_of_100_000_map_points(b: &mut Bencher) {
    //     let mut rng = rand::thread_rng();
    //     b.iter(|| {
    //         for _ in 0..100_000 {
    //             let p1 = random_map_point(&mut rng);
    //             let p2 = random_map_point(&mut rng);
    //             let d = MapPoint::distance(p1, p2);
    //             assert!(d.unwrap().to_meters() >= 0.0);
    //         }
    //     });
    // }
}
