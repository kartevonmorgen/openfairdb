pub use ofdb_core::{db, repositories, usecases, util};

pub mod entities {
    pub use ofdb_core::entities::*;
    #[cfg(test)]
    pub use ofdb_entities::builders::*;
}

pub mod prelude {

    use std::result;

    pub use ofdb_entities::password::Password;

    pub use ofdb_application::error::*;

    pub use super::{
        db::*,
        entities::*,
        repositories::*,
        util::{geo::MapPoint, nonce::Nonce, time::Timestamp},
    };

    pub type Result<T> = result::Result<T, ofdb_application::error::AppError>;
}
