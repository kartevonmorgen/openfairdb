use super::{events::EventQuery, tantivy::SearchEngine, *};
use icalendar::{CalendarDateTime, EventLike};
use std::iter::once;

#[get("/export/events.csv?<query..>")]
pub fn csv_export(
    connections: sqlite::Connections,
    search_engine: SearchEngine,
    auth: Auth,
    query: EventQuery,
) -> result::Result<(ContentType, String), ApiError> {
    let events = query_events(connections, search_engine, auth, query)?;

    let records: Vec<_> = events.map(adapters::csv::EventRecord::from).collect();

    let buffer: Vec<u8> = vec![];
    let mut writer = csv::Writer::from_writer(buffer);

    for r in records {
        writer.serialize(r)?;
    }
    let data = String::from_utf8(writer.into_inner()?)?;

    Ok((ContentType::CSV, data))
}

#[get("/export/entries.csv?<query..>")]
pub fn entries_csv_export(
    connections: sqlite::Connections,
    search_engine: SearchEngine,
    auth: Auth,
    query: search::SearchQuery,
) -> result::Result<(ContentType, String), ApiError> {
    let db = connections.shared()?;

    let moderated_tags = match auth.organization(&db) {
        Ok(org) => org.moderated_tags,
        Err(err) => match err {
            AppError::Business(BError::Parameter(ParameterError::Unauthorized)) => {
                vec![]
            }
            _ => Err(err)?,
        },
    };

    let user = auth.user_with_min_role(&db, Role::Scout)?;

    let (req, limit) = search::parse_search_query(&query)?;
    let limit = if let Some(limit) = limit {
        // Limited
        limit
    } else {
        // Unlimited
        db.count_places()? + 100
    };

    let entries_categories_and_ratings = {
        let all_categories: Vec<_> = db.all_categories()?;
        usecases::search(&db, &*search_engine, req, limit)?
            .0
            .into_iter()
            .filter_map(|indexed_entry| {
                let IndexedPlace {
                    ref id,
                    ref ratings,
                    ..
                } = indexed_entry;
                if let Ok((mut place, _)) = db.get_place(id) {
                    let (tags, categories) = Category::split_from_tags(place.tags);
                    place.tags = tags;
                    let categories = all_categories
                        .iter()
                        .filter(|c1| categories.iter().any(|c2| c1.id == c2.id))
                        .cloned()
                        .collect::<Vec<Category>>();
                    let place = usecases::export_place(
                        place,
                        user.role,
                        moderated_tags
                            .iter()
                            .map(|moderated_tag| moderated_tag.label.as_str()),
                    );
                    Some((place, categories, ratings.total()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    // Release the database connection asap
    drop(db);

    let records: Vec<_> = entries_categories_and_ratings
        .into_iter()
        .map(adapters::csv::CsvRecord::from)
        .collect();

    let buf: Vec<u8> = vec![];
    let mut wtr = csv::Writer::from_writer(buf);

    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok((ContentType::CSV, data))
}

// TODO: make this configurable
const ICAL_CALENDAR_NAME: &str = "OpenFairDB events";

#[get("/export/events.ical?<query..>")]
pub fn events_ical_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    auth: Auth,
    query: EventQuery,
) -> result::Result<(ContentType, String), ApiError> {
    let events = query_events(connections, search_engine, auth, query)?;

    let mut calendar = events
        .filter_map(event_to_ical)
        .collect::<icalendar::Calendar>();
    let calendar = calendar.name(ICAL_CALENDAR_NAME).done();
    let data = calendar.to_string();
    let content_type = ContentType::new("text", "calendar");
    Ok((content_type, data))
}

fn event_to_ical(event: Event) -> Option<icalendar::Event> {
    use icalendar::{Class, Component, Property};

    let Event {
        id,
        title,
        description,
        contact,
        start,
        tags,
        end,
        location,
        ..
    } = event;
    let start_time_as_millis = start.as_millis();
    let naive_dt = chrono::NaiveDateTime::from_timestamp_millis(start_time_as_millis);
    debug_assert!(naive_dt.is_some());

    let start_calendar_date_time = timestamp_as_calendar_date_time(start);
    debug_assert!(start_calendar_date_time.is_some());

    let Some(start_calendar_date_time) = start_calendar_date_time else {
        log::warn!(
            "Invalid start date time for event '{}' (ID={})",
            &title,
            id.as_str()
        );
        return None;
    };

    let mut event = icalendar::Event::new()
  .uid(id.as_str())
  .summary(&title)
  .starts(start_calendar_date_time)
  .class(Class::Public)
  // https://icalendar.org/iCalendar-RFC-5545/3-8-1-2-categories.html
  .append_property(Property::new("CATEGORIES", &tags.join(",")).done())
  .done();

    if let Some(l) = &location {
        // https://icalendar.org/iCalendar-RFC-5545/3-8-1-6-geographic-position.html
        event = event
            .append_property(
                Property::new(
                    "GEO",
                    &format!("{};{}", l.pos.lat().to_deg(), l.pos.lng().to_deg()),
                )
                .done(),
            )
            .done();

        if let Some(addr) = address_string_from_location(l) {
            // https://icalendar.org/iCalendar-RFC-5545/3-8-1-6-geographic-position.html
            event = event
                .append_property(Property::new("LOCATION", &addr).done())
                .done();
        }
    }

    if let Some(contact) = contact_to_string(&contact) {
        // https://icalendar.org/iCalendar-RFC-5545/3-8-4-2-contact.html
        event = event
            .append_property(Property::new("CONTACT", &contact).done())
            .done();
    }
    if let Some(end) = end {
        let end_calendar_date_time = timestamp_as_calendar_date_time(end);
        debug_assert!(end_calendar_date_time.is_some());
        let Some(end) = end_calendar_date_time else {
                log::warn!(
                    "Invalid end date time for event '{}' (ID={})",
                    &title,
                    id.as_str()
                );
                return None;
        };
        event = event.ends(end).done();
    }
    if let Some(desc) = description {
        event = event.description(&desc).done();
    }
    Some(event)
}

fn timestamp_as_calendar_date_time(ts: Timestamp) -> Option<CalendarDateTime> {
    let unix_timestamp_in_millis = ts.as_millis();
    let naive_dt = chrono::NaiveDateTime::from_timestamp_millis(unix_timestamp_in_millis)?;
    Some(CalendarDateTime::Floating(naive_dt))
}

fn query_events(
    connections: sqlite::Connections,
    search_engine: SearchEngine,
    auth: Auth,
    query: EventQuery,
) -> result::Result<impl Iterator<Item = Event>, ApiError> {
    let query = query.into_inner();
    let db = connections.shared()?;

    let moderated_tags = if let Ok(org) = auth.organization(&db) {
        org.moderated_tags
    } else {
        vec![]
    };

    let user = auth.user_with_min_role(&db, Role::Scout)?;

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
    let events = usecases::query_events(&db, &*search_engine, query)?;
    // Release the database connection asap
    drop(db);

    let events = events.into_iter().map(move |event| {
        usecases::export_event(
            event,
            user.role,
            moderated_tags
                .iter()
                .map(|moderated_tag| moderated_tag.label.as_str()),
        )
    });
    Ok(events)
}

fn address_string_from_location(l: &Location) -> Option<String> {
    let address = l.address.as_ref()?;
    if address.is_empty() {
        return None;
    }

    let Address {
        street,
        zip,
        city,
        country,
        state,
        ..
    } = address;

    Some(
        once(street)
            .chain(once(zip))
            .chain(once(city))
            .chain(once(country))
            .chain(once(state))
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
            .join("\\, "),
    )
}

fn contact_to_string(contact: &Option<Contact>) -> Option<String> {
    let contact = contact.as_ref()?;
    if contact.is_empty() {
        return None;
    }

    let Contact {
        name, email, phone, ..
    } = contact.clone();

    Some(
        once(name)
            .chain(once(email.map(|m| m.to_string())))
            .chain(once(phone))
            .flatten()
            .collect::<Vec<_>>()
            .join("\\, "),
    )
}
