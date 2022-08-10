use super::{events::EventQuery, *};

#[get("/export/events.csv?<query..>")]
pub fn csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    auth: Auth,
    query: EventQuery,
) -> result::Result<(ContentType, String), ApiError> {
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

    Ok((ContentType::CSV, data))
}

#[get("/export/entries.csv?<query..>")]
pub fn entries_csv_export(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    auth: Auth,
    query: search::SearchQuery,
) -> result::Result<(ContentType, String), ApiError> {
    let db = connections.shared()?;

    let moderated_tags = match auth.organization(&db) {
        Ok(org) => org.moderated_tags,
        _ => vec![],
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
