use rocket::http::Header;

use super::{super::tests::prelude::*, *};

fn now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

mod archive;
mod create;
mod delete;
mod export_csv;
mod read;
mod update;
