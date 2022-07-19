pub mod db;
pub mod entities;
pub mod error;
pub mod repositories;
pub mod usecases;

pub use ofdb_core::util;

pub mod prelude {

    use std::result;

    pub use ofdb_entities::password::Password;

    pub use super::{
        db::*,
        entities::*,
        error::*,
        repositories::*,
        util::{
            geo::{Distance, LatCoord, LngCoord, MapPoint},
            nonce::Nonce,
            time::Timestamp,
        },
    };

    pub type Result<T> = result::Result<T, super::error::Error>;
}
