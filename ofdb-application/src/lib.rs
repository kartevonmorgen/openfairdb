#[macro_use]
extern crate log;

mod archive_comments;
mod archive_events;
mod archive_ratings;
mod change_user_role;
mod create_event;
mod create_place;
mod create_rating;
mod reset_password;
mod review_place_with_token;
mod review_places;
mod send_update_reminders;
mod update_event;
mod update_place;

pub mod prelude {
    pub use super::{
        archive_comments::*, archive_events::*, archive_ratings::*, change_user_role::*,
        create_event::*, create_place::*, create_rating::*, reset_password::*,
        review_place_with_token::*, review_places::*, send_update_reminders::*, update_event::*,
        update_place::*,
    };
}

pub mod error;

pub type Result<T> = std::result::Result<T, error::AppError>;

pub(crate) use ofdb_core::{db::*, entities::*, repositories::*, usecases};

#[cfg(test)]
pub(crate) mod tests;

pub(crate) mod sqlite {
    pub use ofdb_db_sqlite::Connections;
}
