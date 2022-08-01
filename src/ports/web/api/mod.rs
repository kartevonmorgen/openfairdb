use std::{fmt::Display, result};

use ofdb_boundary::Error as JsonErrorResponse;
use rocket::serde::json::{Error as JsonError, Json};
use rocket::{
    self, delete, get,
    http::{ContentType, Cookie, CookieJar, Status},
    post,
    response::{self, Responder},
    routes, Request, Route, State,
};

use super::guards::*;
use crate::{
    adapters::{
        self,
        json::{self, to_json},
    },
    core::{
        error::Error,
        prelude::*,
        usecases,
        util::{geo, split_ids},
    },
    infrastructure::{db::tantivy, error::AppError, flows::prelude as flows},
    ports::web::{jwt, notify::*, sqlite},
};
use ofdb_core::repositories::Error as RepoError;
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
#[cfg(test)]
pub mod tests;
mod users;
mod util;
use self::error::Error as ApiError;

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

impl<'r, 'o: 'r> Responder<'r, 'o> for AppError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        if let AppError::Business(err) = &self {
            match err {
                Error::Parameter(ref err) => {
                    return match *err {
                        ParameterError::Credentials | ParameterError::Unauthorized => {
                            json_error_response(req, err, Status::Unauthorized)
                        }
                        ParameterError::Forbidden
                        | ParameterError::ModeratedTag
                        | ParameterError::EmailNotConfirmed => {
                            json_error_response(req, err, Status::Forbidden)
                        }
                        _ => json_error_response(req, err, Status::BadRequest),
                    };
                }
                Error::Repo(RepoError::NotFound) => {
                    return json_error_response(req, err, Status::NotFound);
                }
                _ => {}
            }
        }
        error!("Error: {}", self);
        Err(Status::InternalServerError)
    }
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
