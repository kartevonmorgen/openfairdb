use super::*;
use crate::{
    adapters::json::from_json,
    core::{
        prelude::Result as CoreResult,
        util::{self, geo::MapBbox, validate},
    },
};
use ofdb_application::error::BError as Error;
use ofdb_boundary::NewEvent;
use ofdb_core::gateways::geocode::GeoCodingGateway;
use rocket::{
    delete,
    form::{self, DataField, FromForm, ValueField},
    get,
    http::Status as HttpStatus,
    post, put,
};

#[cfg(test)]
mod tests;

fn check_and_set_address_location<G>(e: &mut usecases::NewEvent, geo_gw: &G) -> Option<MapPoint>
where
    G: GeoCodingGateway + ?Sized,
{
    let pos = if let (Some(lat), Some(lng)) = (e.lat, e.lng) {
        MapPoint::try_from_lat_lng_deg(lat, lng)
            .map(Some)
            .unwrap_or_default()
    } else {
        None
    };
    if pos.unwrap_or_default().is_valid() {
        // Preserve valid geo locations
        return pos;
    }
    // TODO: Parse logical parts of NewEvent earlier
    let addr = Address {
        street: e.street.clone(),
        zip: e.zip.clone(),
        city: e.city.clone(),
        country: e.country.clone(),
        state: e.state.clone(),
    };

    geo_gw
        .resolve_address_lat_lng(&addr)
        .and_then(|(lat, lng)| {
            if let Ok(pos) = MapPoint::try_from_lat_lng_deg(lat, lng) {
                log::debug!(
                    "Updating event location: ({:?}, {:?}) -> {:?}",
                    e.lat,
                    e.lng,
                    pos
                );
                e.lat = Some(lat);
                e.lng = Some(lng);
            }
            pos
        })
}

#[post("/events", format = "application/json", data = "<ev>")]
pub fn post_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    notify: &State<Notify>,
    auth: Auth,
    ev: JsonResult<NewEvent>,
    geo_gw: &State<GeoCoding>,
) -> Result<String> {
    let org = auth.organization(&connections.shared()?)?;
    let mut new_event = from_json::try_new_event(ev?.into_inner())?;
    check_and_set_address_location(&mut new_event, &*geo_gw.0);
    let event = flows::create_event(
        &connections,
        &mut *search_engine,
        &*notify.0,
        Some(&org.api_token),
        new_event,
    )?;
    Ok(Json(event.id.to_string()))
}

#[post("/events", format = "application/json", data = "<_e>", rank = 2)]
// NOTE:
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn post_event(mut _db: sqlite::Connections, _e: JsonResult<NewEvent>) -> HttpStatus {
    HttpStatus::Unauthorized
}
// But in the future we might allow anonymous event creation:
//
// pub fn post_event(mut db: sqlite::Connections, e: Json<usecases::NewEvent>)
// -> Result<String> {     let mut e = e.into_inner();
//     e.created_by = None; // ignore because of missing authorization
//     e.token = None; // ignore token
//     let id = flows::create_event(&*db, &search_engine, e.clone())?;
//     Ok(Json(id))
// }

#[get("/events/<id>")]
pub fn get_event(db: sqlite::Connections, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&db.shared()?, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[put("/events/<_id>", format = "application/json", data = "<_e>", rank = 2)]
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn put_event(mut _db: sqlite::Connections, _id: &str, _e: JsonResult<NewEvent>) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[put("/events/<id>", format = "application/json", data = "<ev>")]
pub fn put_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    notify: &State<Notify>,
    auth: Auth,
    id: &str,
    ev: JsonResult<NewEvent>,
    geo_gw: &State<GeoCoding>,
) -> Result<()> {
    let org = auth.organization(&connections.shared()?)?;
    let mut new_event = from_json::try_new_event(ev?.into_inner())?;
    check_and_set_address_location(&mut new_event, &*geo_gw.0);
    flows::update_event(
        &connections,
        &mut *search_engine,
        &*notify.0,
        Some(&org.api_token),
        id.to_string().into(),
        new_event,
    )?;
    Ok(Json(()))
}

pub struct EventQueryContext<'r> {
    query: usecases::EventQuery,
    errors: form::Errors<'r>,
}

pub struct EventQuery(usecases::EventQuery);

impl EventQuery {
    pub fn into_inner(self) -> usecases::EventQuery {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> FromForm<'r> for EventQuery {
    type Context = EventQueryContext<'r>;

    // TODO: use Options
    fn init(_: form::Options) -> Self::Context {
        Self::Context {
            query: Default::default(),
            errors: form::Errors::new(),
        }
    }

    fn push_value(ctx: &mut Self::Context, field: ValueField<'r>) {
        // TODO: improve error messages
        // TODO: check duplicate fields

        let ValueField { name, value } = field;
        use form::error::{Error, ErrorKind};
        match name.as_name().as_str() {
            "created_by" => {
                match value
                    .parse::<EmailAddress>()
                    .map_err(|_| Error::from(ErrorKind::Validation("Invalid email address".into())))
                {
                    Ok(email) => {
                        ctx.query.created_by = Some(email);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "bbox" => {
                let result = value
                    .parse::<MapBbox>()
                    .map_err(|_| ())
                    .and_then(|bbox| {
                        if !validate::is_valid_bbox(&bbox) {
                            Err(())
                        } else {
                            Ok(bbox)
                        }
                    })
                    .map_err(|_| Error::from(ErrorKind::Validation("Invalid bounding box".into())));

                match result {
                    Ok(bbox) => {
                        ctx.query.bbox = Some(bbox);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "limit" => {
                let result = value.parse().map_err(Error::from).and_then(|limit| {
                    validate_and_adjust_query_limit(limit)
                        .map_err(|_| Error::from(ErrorKind::Validation("Invalid limit".into())))
                });
                match result {
                    Ok(limit) => {
                        ctx.query.limit = Some(limit);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "start_max" => {
                let result = value.parse().map_err(Error::from).and_then(|seconds| {
                    Timestamp::try_from_secs(seconds).map_err(|_| {
                        Error::from(ErrorKind::Validation(
                            "Invalid value for 'start_max'".into(),
                        ))
                    })
                });
                match result {
                    Ok(max) => {
                        ctx.query.start_max = Some(max);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "start_min" => {
                let result = value.parse().map_err(Error::from).and_then(|seconds| {
                    Timestamp::try_from_secs(seconds).map_err(|_| {
                        Error::from(ErrorKind::Validation(
                            "Invalid value for 'start_min'".into(),
                        ))
                    })
                });
                match result {
                    Ok(min) => {
                        ctx.query.start_min = Some(min);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "end_max" => {
                let result = value.parse().map_err(Error::from).and_then(|seconds| {
                    Timestamp::try_from_secs(seconds).map_err(|_| {
                        Error::from(ErrorKind::Validation("Invalid value for 'end_max'".into()))
                    })
                });
                match result {
                    Ok(max) => {
                        ctx.query.end_max = Some(max);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "end_min" => {
                let result = value.parse().map_err(Error::from).and_then(|seconds| {
                    Timestamp::try_from_secs(seconds).map_err(|_| {
                        Error::from(ErrorKind::Validation("Invalid value for 'end_min'".into()))
                    })
                });
                match result {
                    Ok(min) => {
                        ctx.query.end_min = Some(min);
                    }
                    Err(err) => {
                        ctx.errors.push(err.with_name(name));
                    }
                }
            }
            "tag" => {
                if !value.is_empty() {
                    let tag = value.to_string();
                    ctx.query.tags.get_or_insert(vec![]).push(tag);
                }
            }
            "text" => {
                if !value.is_empty() {
                    ctx.query.text = Some(value.to_string());
                }
            }
            name => {
                ctx.errors
                    .push(Error::from(ErrorKind::Unexpected).with_name(name));
            }
        }
    }

    async fn push_data(ctx: &mut Self::Context, field: DataField<'r, '_>) {
        use form::error::{Error, ErrorKind};
        ctx.errors
            .push(Error::from(ErrorKind::Unexpected).with_name(field.name));
    }

    fn finalize(this: Self::Context) -> form::Result<'r, Self> {
        if this.errors.is_empty() {
            Ok(EventQuery(this.query))
        } else {
            Err(this.errors)
        }
    }
}

const MAX_RESULT_LIMIT: usize = 2000;

#[allow(clippy::absurd_extreme_comparisons)]
fn validate_and_adjust_query_limit(limit: usize) -> CoreResult<usize> {
    if limit > MAX_RESULT_LIMIT {
        info!(
            "Requested limit {} exceeds maximum limit {} for event search results",
            limit, MAX_RESULT_LIMIT
        );
        Ok(MAX_RESULT_LIMIT)
    } else if limit <= 0 {
        warn!("Invalid search limit: {}", limit);
        Err(Error::Parameter(ParameterError::InvalidLimit).into())
    } else {
        Ok(limit)
    }
}

#[get("/events?<query..>")]
pub fn get_events_with_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    auth: Auth,
    query: EventQuery,
) -> Result<Vec<json::Event>> {
    let db = connections.shared()?;
    let org = match auth.organization(&db) {
        Ok(org) => org,
        Err(AppError::Business(Error::Parameter(ParameterError::Unauthorized))) => {
            drop(db);
            return get_events_chronologically(connections, search_engine, query);
        }
        Err(e) => return Err(e.into()),
    };
    let events = usecases::query_events(&db, &*search_engine, query.into_inner())?;
    // Release the database connection asap
    drop(db);

    let moderated_tags = org.moderated_tags;
    let events: Vec<_> = events
        .into_iter()
        .map(|e| {
            usecases::filter_event(
                e,
                moderated_tags
                    .iter()
                    .map(|moderated_tag| moderated_tag.label.as_str()),
            )
        })
        .map(json::Event::from)
        .collect();

    Ok(Json(events))
}

#[get("/events?<query..>", rank = 2)]
pub fn get_events_chronologically(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    query: EventQuery,
) -> Result<Vec<json::Event>> {
    let query = query.into_inner();
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }

    let db = connections.shared()?;
    let events = usecases::query_events(&db, &*search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let moderated_tags = [];
    let events: Vec<_> = events
        .into_iter()
        .map(|e| usecases::filter_event(e, moderated_tags.iter().map(String::as_str)))
        .map(json::Event::from)
        .collect();

    Ok(Json(events))
}

#[post("/events/<ids>/archive")]
pub fn post_events_archive(
    auth: Auth,
    db: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    ids: String,
) -> StatusResult {
    let ids = util::split_ids(&ids);
    if ids.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyIdList).into());
    }
    let archived_by_email = {
        let db = db.shared()?;
        // Only scouts and admins are entitled to review events
        auth.user_with_min_role(&db, Role::Scout)?.email
    };
    let update_count = flows::archive_events(&db, &mut *search_engine, &ids, &archived_by_email)?;
    if update_count < ids.len() {
        log::info!(
            "Archived only {} of {} event(s): {:?}",
            update_count,
            ids.len(),
            ids
        );
    }
    Ok(HttpStatus::NoContent)
}

#[delete("/events/<_id>", rank = 2)]
pub fn delete_event(mut _db: sqlite::Connections, _id: &str) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[delete("/events/<id>")]
pub fn delete_event_with_token(db: sqlite::Connections, auth: Auth, id: &str) -> StatusResult {
    let org = auth.organization(&db.shared()?)?;
    usecases::delete_event(&db.exclusive()?, &org.api_token, id)?;
    // TODO: Replace with HttpStatus::NoContent
    Ok(HttpStatus::Ok)
}
