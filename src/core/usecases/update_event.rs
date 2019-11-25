use super::{try_into_new_event, NewEventMode};
use crate::core::prelude::*;

pub use super::NewEvent as UpdateEvent;

pub fn update_event<D: Db>(
    db: &mut D,
    token: Option<&str>,
    id: &str,
    e: UpdateEvent,
) -> Result<()> {
    let mut updated_event = try_into_new_event(db, token, e, NewEventMode::Update(id))?;
    debug!("Updating event: {:?}", updated_event);
    updated_event.id = id.into();
    db.update_event(&updated_event)?;
    Ok(())
}
