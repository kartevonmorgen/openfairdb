use crate::core::prelude::*;

pub fn query_events<D: Db>(
    db: &D,
    tags: Option<Vec<String>>,
    created_by: &Option<String>,
) -> Result<Vec<Event>> {
    let events = db.all_events()?;
    Ok(events)
}
