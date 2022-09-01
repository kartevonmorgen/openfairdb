use super::*;

#[get("/server/version")]
pub fn get_version(version: &State<Version>) -> &'static str {
    version.0
}

#[get("/server/openapi.yaml")]
pub fn get_api() -> (ContentType, &'static str) {
    let data = include_str!("../../../../openapi.yaml");
    let c_type = ContentType::new("text", "yaml");
    (c_type, data)
}

#[get("/duplicates/<ids>")]
pub fn get_duplicates(
    connections: sqlite::Connections,
    search_engine: tantivy::SearchEngine,
    ids: String,
) -> Result<Vec<(String, String, json::DuplicateType)>> {
    let ids = split_ids(&ids);
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let places = connections.shared()?.get_places(&ids)?;
    let results = usecases::find_duplicates(&*search_engine, &places)?;
    Ok(Json(
        results
            .into_iter()
            .map(|(id1, id2, dup)| {
                (
                    id1.to_string(),
                    id2.to_string(),
                    to_json::duplicate_type(dup),
                )
            })
            .collect(),
    ))
}

#[get("/tags")]
pub fn get_tags(connections: sqlite::Connections) -> Result<Vec<String>> {
    let tags = connections.shared()?.all_tags()?;
    Ok(Json(tags.into_iter().map(|t| t.id).collect()))
}

#[get("/categories")]
pub fn get_categories(connections: sqlite::Connections) -> Result<Vec<json::Category>> {
    let categories = connections
        .shared()?
        .all_categories()?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(categories))
}

#[get("/categories/<ids>")]
pub fn get_category(connections: sqlite::Connections, ids: String) -> Result<Vec<json::Category>> {
    // TODO: Only lookup and return a single entity
    // TODO: Add a new method for searching multiple ids
    let uids = split_ids(&ids);
    if uids.is_empty() {
        return Ok(Json(vec![]));
    }
    let categories = connections
        .shared()?
        .all_categories()?
        .into_iter()
        .filter(|c| uids.iter().any(|id| c.id.as_str() == *id))
        .map(Into::into)
        .collect();
    Ok(Json(categories))
}
