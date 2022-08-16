use crate::{address::*, geo::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub pos: MapPoint,
    pub address: Option<Address>,
}
