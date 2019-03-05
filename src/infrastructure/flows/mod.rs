mod add_entry;
mod add_rating;
mod archive_comments;
mod archive_entries;
mod update_entry;

pub mod prelude {
    pub use super::{
        add_entry::*, add_entry::*, add_rating::*, archive_comments::*, archive_entries::*, update_entry::*,
    };
}

pub type Result<T> = std::result::Result<T, error::AppError>;

pub(crate) use super::{db::sqlite, error, notify};

pub(crate) use crate::core::{prelude::*, usecases};
