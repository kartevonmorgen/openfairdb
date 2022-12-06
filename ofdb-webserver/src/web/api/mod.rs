use std::{fmt::Display, result};

use ofdb_boundary::Error as JsonErrorResponse;
use rocket::serde::json::{Error as JsonError, Json};
use rocket::{
    self, delete, get,
    http::{ContentType, Cookie, CookieJar, Status},
    post,
    response::{self, Responder},
    routes, Route, State,
};

use super::guards::*;
use crate::{
    adapters::{
        self,
        json::{self, to_json},
    },
    core::{
        prelude::*,
        usecases,
        util::{geo, split_ids},
    },
    web::{jwt, sqlite, tantivy},
};
use ofdb_application::{error::AppError, error::BError as Error, prelude as flows};
use ofdb_core::usecases::Error as ParameterError;

pub mod captcha;
mod count;
mod entries;
mod error;
pub mod events;
mod export;
mod places;
mod ratings;
mod search;
mod subscriptions;
mod users;
mod util;

pub use self::error::Error as ApiError;

#[cfg(test)]
pub mod tests;

type Result<T> = result::Result<Json<T>, ApiError>;
type JsonResult<'a, T> = result::Result<Json<T>, JsonError<'a>>;
type StatusResult = result::Result<Status, ApiError>;

pub fn routes() -> Vec<Route> {
    routes![
        // ---   search   --- //
        search::get_search,
        search::post_search_duplicates,
        // ---   entries   --- //
        entries::get_entry,
        entries::get_entries_recently_changed,
        entries::get_entries_most_popular_tags,
        entries::post_entry,
        entries::put_entry,
        // ---   places   --- //
        places::count_pending_clearances,
        places::list_pending_clearances,
        places::post_review,
        places::update_pending_clearances,
        places::get_place,
        places::get_place_history,
        places::get_place_history_revision,
        places::get_not_updated,
        // ---   events   --- //
        events::post_event,
        events::post_event_with_token,
        events::get_event,
        events::get_events_chronologically,
        events::get_events_with_token,
        events::put_event,
        events::put_event_with_token,
        events::post_events_archive,
        events::delete_event,
        events::delete_event_with_token,
        // ---   users   --- //
        users::post_login,
        users::post_logout,
        users::confirm_email_address,
        users::post_request_password_reset,
        users::post_reset_password,
        users::post_user,
        users::get_user,
        users::get_current_user,
        users::delete_user,
        // ---   subscriptions   --- //
        subscriptions::subscribe_to_bbox,
        subscriptions::get_bbox_subscriptions,
        subscriptions::unsubscribe_all_bboxes,
        // ---   export   --- //
        export::csv_export,
        export::entries_csv_export,
        export::events_ical_export,
        // ---   ratings   --- //
        ratings::post_rating,
        ratings::load_rating,
        // ---   count   --- //
        count::get_count_entries,
        count::get_count_tags,
        util::get_version,
        util::get_api,
        util::get_duplicates,
        util::get_categories,
        util::get_category,
        util::get_tags,
        // ---   captcha   --- //
        captcha::post_captcha,
        captcha::get_captcha,
        captcha::post_captcha_verify,
    ]
}

fn json_error_response<'r, 'o: 'r, E: Display>(
    req: &'r rocket::Request<'_>,
    err: &E,
    status: Status,
) -> response::Result<'o> {
    let message = err.to_string();
    let boundary_error = JsonErrorResponse {
        http_status: status.code,
        message,
    };
    Json(boundary_error).respond_to(req).map(|mut res| {
        res.set_status(status);
        res
    })
}
