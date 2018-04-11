pub mod db;
pub mod entities;
pub mod error;
pub mod usecases;
pub mod util;

pub mod prelude {

    use std::result;

    pub use super::db::*;
    pub use super::entities::*;
    pub use super::error::*;

    pub type Result<T> = result::Result<T, super::error::Error>;
}
