use super::{index_event, try_into_new_event, NewEventMode};
use crate::core::prelude::*;

pub use super::NewEvent as UpdateEvent;

pub fn update_event<D: Db>(
    db: &D,
    indexer: &mut dyn EventIndexer,
    token: Option<&str>,
    id: &str,
    e: UpdateEvent,
) -> Result<()> {
    let event = try_into_new_event(db, token, e, NewEventMode::Update(id))?;
    debug_assert_eq!(event.id, Id::from(id));

    debug!("Updating event: {:?}", event);
    db.update_event(&event)?;

    // Index newly added event
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = index_event(indexer, &event).and_then(|_| indexer.flush_index()) {
        error!("Failed to index newly added event {}: {}", event.id, err);
    }

    Ok(())
}
