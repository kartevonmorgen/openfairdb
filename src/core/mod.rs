pub mod db;
pub mod entities;
pub mod error;
pub mod repositories;
pub mod usecases;
pub mod util;

pub mod prelude {

    use std::result;

    pub use super::db::*;
    pub use super::entities::*;
    pub use super::error::*;
    pub use super::repositories::*;
    pub use super::util::{
        geo::{Distance, LatCoord, LngCoord, MapPoint},
        nonce::Nonce,
        password::Password,
        time::{Timestamp, TimestampMs},
    };

    pub type Result<T> = result::Result<T, super::error::Error>;
}
