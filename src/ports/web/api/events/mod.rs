use super::{super::guards::Bearer, *};
use crate::{
    adapters,
    core::{
        prelude::Result as CoreResult,
        util::{geo::MapBbox, validate},
    },
    infrastructure::{flows::prelude as flows, GEO_CODING_GW},
};
use ofdb_core::gateways::geocode::GeoCodingGateway;

use rocket::{
    http::{RawStr, Status as HttpStatus},
    request::{FromQuery, Query},
};

#[cfg(test)]
mod tests;

fn check_and_set_address_location(e: &mut usecases::NewEvent) -> Option<MapPoint> {
    let pos = if let (Some(lat), Some(lng)) = (e.lat, e.lng) {
        MapPoint::try_from_lat_lng_deg(lat, lng)
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

    GEO_CODING_GW
        .resolve_address_lat_lng(&addr)
        .and_then(|(lat, lng)| {
            let pos = MapPoint::try_from_lat_lng_deg(lat, lng);
            if pos.unwrap_or_default().is_valid() {
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

#[post("/events", format = "application/json", data = "<e>")]
pub fn post_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    notify: Notify,
    token: Bearer,
    e: Json<usecases::NewEvent>,
) -> Result<String> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    let event = flows::create_event(
        &connections,
        &mut search_engine,
        &*notify,
        Some(&token.0),
        e,
    )?;
    Ok(Json(event.id.to_string()))
}

#[post("/events", format = "application/json", data = "<_e>", rank = 2)]
// NOTE:
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn post_event(mut _db: sqlite::Connections, _e: Json<usecases::NewEvent>) -> HttpStatus {
    HttpStatus::Unauthorized
}
// But in the future we might allow anonymous event creation:
//
// pub fn post_event(mut db: sqlite::Connections, e: Json<usecases::NewEvent>) -> Result<String> {
//     let mut e = e.into_inner();
//     e.created_by = None; // ignore because of missing authorization
//     e.token = None; // ignore token
//     let id = flows::create_event(&*db, &search_engine, e.clone())?;
//     Ok(Json(id))
// }

#[get("/events/<id>")]
pub fn get_event(db: sqlite::Connections, id: String) -> Result<json::Event> {
    let mut ev = usecases::get_event(&*db.shared()?, &id)?;
    ev.created_by = None; // don't show creators email to unregistered users
    Ok(Json(ev.into()))
}

#[put("/events/<_id>", format = "application/json", data = "<_e>", rank = 2)]
// At the moment we don't want to allow anonymous event creation.
// So for now we assure that it's blocked:
pub fn put_event(
    mut _db: sqlite::Connections,
    _id: &RawStr,
    _e: Json<usecases::NewEvent>,
) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[put("/events/<id>", format = "application/json", data = "<e>")]
pub fn put_event_with_token(
    connections: sqlite::Connections,
    mut search_engine: tantivy::SearchEngine,
    notify: Notify,
    token: Bearer,
    id: &RawStr,
    e: Json<usecases::NewEvent>,
) -> Result<()> {
    let mut e = e.into_inner();
    check_and_set_address_location(&mut e);
    flows::update_event(
        &connections,
        &mut search_engine,
        &*notify,
        Some(&token.0),
        id.to_string().into(),
        e,
    )?;
    Ok(Json(()))
}

impl<'q> FromQuery<'q> for usecases::EventQuery {
    type Error = crate::core::prelude::Error;

    fn from_query(query: Query<'q>) -> std::result::Result<Self, Self::Error> {
        let created_by = query
            .clone()
            .filter(|i| i.key == "created_by")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
            .map(|s| s.parse::<Email>())
            .transpose()
            .map_err(|_| ParameterError::Email)?;

        let bbox = if let Some(bbox) = query
            .clone()
            .filter(|i| i.key == "bbox")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            let bbox = bbox
                .parse::<MapBbox>()
                .map_err(|_err| ParameterError::Bbox)?;
            validate::bbox(&bbox)?;
            Some(bbox)
        } else {
            None
        };

        let limit = if let Some(limit) = query
            .clone()
            .filter(|i| i.key == "limit")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(validate_and_adjust_query_limit(limit.parse()?)?)
        } else {
            None
        };

        let start_max = if let Some(start_max) = query
            .clone()
            .filter(|i| i.key == "start_max")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(Timestamp::from_inner(start_max.parse()?))
        } else {
            None
        };

        let start_min = if let Some(start_min) = query
            .clone()
            .filter(|i| i.key == "start_min")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty())
        {
            Some(Timestamp::from_inner(start_min.parse()?))
        } else {
            None
        };

        let tags: Vec<_> = query
            .clone()
            .filter(|i| i.key == "tag")
            .map(|i| i.value.to_string())
            .filter(|v| !v.is_empty())
            .collect();
        let tags = if tags.is_empty() { None } else { Some(tags) };

        let text = query
            .clone()
            .filter(|i| i.key == "text")
            .map(|i| i.value.url_decode_lossy())
            .find(|v| !v.is_empty());

        drop(query); // silence clippy warning
        Ok(usecases::EventQuery {
            bbox,
            created_by,
            limit,
            start_max,
            start_min,
            tags,
            text,
        })
    }
}

const MAX_RESULT_LIMIT: usize = 500;

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
        Err(Error::Parameter(ParameterError::InvalidLimit))
    } else {
        Ok(limit)
    }
}

#[get("/events?<query..>")]
pub fn get_events_with_token(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    token: Bearer,
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    let db = connections.shared()?;
    let org = usecases::authorize_organization_by_api_token(&*db, &token.0)?;
    let events = usecases::query_events(&*db, &search_engine, query)?;
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
    query: usecases::EventQuery,
) -> Result<Vec<json::Event>> {
    if query.created_by.is_some() {
        return Err(Error::Parameter(ParameterError::Unauthorized).into());
    }

    let db = connections.shared()?;
    let events = usecases::query_events(&*db, &search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let moderated_tags = vec![];
    let events: Vec<_> = events
        .into_iter()
        .map(|e| usecases::filter_event(e, moderated_tags.iter().map(String::as_str)))
        .map(json::Event::from)
        .collect();

    Ok(Json(events))
}

#[get("/export/events.csv?<query..>")]
pub fn csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    bearer: Option<Bearer>,
    login: Login,
    query: usecases::EventQuery,
) -> result::Result<Content<String>, AppError> {
    let db = connections.shared()?;

    let moderated_tags = if let Some(bearer) = bearer {
        let api_token = bearer.0;
        let org = usecases::authorize_organization_by_api_token(&*db, &api_token)?;
        org.moderated_tags
    } else {
        vec![]
    };

    let user = usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?;

    let limit = if let Some(limit) = query.limit {
        // Limited
        limit
    } else {
        // Unlimited
        db.count_events()? + 100
    };
    let query = usecases::EventQuery {
        limit: Some(limit),
        ..query
    };
    let events = usecases::query_events(&*db, &search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let events = events.into_iter().map(|e| {
        usecases::export_event(
            e,
            user.role,
            moderated_tags
                .iter()
                .map(|moderated_tag| moderated_tag.label.as_str()),
        )
    });

    let records: Vec<_> = events.map(adapters::csv::EventRecord::from).collect();

    let buff: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buff);

    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(Content(ContentType::CSV, data))
}

#[post("/events/<ids>/archive")]
pub fn post_events_archive(
    login: Login,
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
        usecases::authorize_user_by_email(&*db, &login.0, Role::Scout)?.email
    };
    let update_count = flows::archive_events(&db, &mut search_engine, &ids, &archived_by_email)?;
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
pub fn delete_event(mut _db: sqlite::Connections, _id: &RawStr) -> HttpStatus {
    HttpStatus::Unauthorized
}

#[delete("/events/<id>")]
pub fn delete_event_with_token(
    db: sqlite::Connections,
    token: Bearer,
    id: &RawStr,
) -> StatusResult {
    usecases::delete_event(&mut *db.exclusive()?, &token.0, &id.to_string())?;
    // TODO: Replace with HttpStatus::NoContent
    Ok(HttpStatus::Ok)
}
