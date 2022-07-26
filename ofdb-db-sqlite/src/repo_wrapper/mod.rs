use super::*;

mod read_only;
mod read_write;

pub use read_only::*;
pub use read_write::*;

type Result<T> = std::result::Result<T, ofdb_core::RepoError>;
use ofdb_core::entities::*;
use ofdb_core::repositories::*;
