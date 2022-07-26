use super::prelude::*;

pub fn get_event<R: EventRepo>(repo: &R, id: &str) -> Result<Event> {
    Ok(repo.get_event(id)?)
}
