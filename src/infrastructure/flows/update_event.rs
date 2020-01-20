use super::*;

use crate::core::error::RepoError;

use diesel::Connection;

pub fn update_event(
    connections: &sqlite::Connections,
    indexer: &mut dyn EventIndexer,
    token: Option<&str>,
    id: Id,
    new_event: usecases::NewEvent,
) -> Result<Event> {
    // Create and add new event
    let event = {
        let connection = connections.exclusive()?;
        let mut prepare_err = None;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                match usecases::import_new_event(
                    &*connection,
                    token,
                    new_event,
                    usecases::NewEventMode::Update(id.as_str()),
                ) {
                    Ok(storable) => {
                        let event = usecases::store_updated_event(&*connection, storable).map_err(
                            |err| {
                                warn!("Failed to store updated event: {}", err);
                                diesel::result::Error::RollbackTransaction
                            },
                        )?;
                        Ok(event)
                    }
                    Err(err) => {
                        prepare_err = Some(err);
                        Err(diesel::result::Error::RollbackTransaction)
                    }
                }
            })
            .map_err(|err| {
                if let Some(err) = prepare_err {
                    err
                } else {
                    RepoError::from(err).into()
                }
            })
    }?;

    // Index newly added event
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::index_event(indexer, &event).and_then(|_| indexer.flush_index()) {
        error!("Failed to re-index updated event {}: {}", event.id, err);
    }

    // Send subscription e-mails
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = notify_event_updated(connections, &event) {
        error!(
            "Failed to send notifications for updated event {}: {}",
            event.id, err
        );
    }

    Ok(event)
}

fn notify_event_updated(connections: &sqlite::Connections, event: &Event) -> Result<()> {
    if let Some(ref location) = event.location {
        let _email_addresses = {
            let conn = connections.shared()?;
            usecases::email_addresses_by_coordinate(&*conn, location.pos)?
        };
        error!("TODO: notify::event_updated {:?}", event);
        //notify::event_updated(&email_addresses, event, all_categories);
    }
    Ok(())
}
