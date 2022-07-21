use ofdb_core::gateways::notify::NotificationGateway;

use super::*;

pub fn create_event(
    connections: &sqlite::Connections,
    indexer: &mut dyn EventIndexer,
    notify: &dyn NotificationGateway,
    token: Option<&str>,
    new_event: usecases::NewEvent,
) -> Result<Event> {
    // Create and add new event
    let event = {
        let connection = connections.exclusive()?;
        let mut prepare_err = None;
        connection
            .transaction::<_, _>(|| {
                match usecases::import_new_event(
                    &connection,
                    token,
                    new_event,
                    usecases::NewEventMode::Create,
                ) {
                    Ok(storable) => {
                        let event = usecases::store_created_event(&connection, storable).map_err(
                            |err| {
                                warn!("Failed to store newly created event: {}", err);
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
                    from_diesel_err(err).into()
                }
            })
    }?;

    // Index newly added event
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::index_event(indexer, &event).and_then(|_| indexer.flush_index()) {
        error!("Failed to index newly added event {}: {}", event.id, err);
    }

    // Send subscription e-mails
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = notify_event_created(connections, notify, &event) {
        error!(
            "Failed to send notifications for newly added event {}: {}",
            event.id, err
        );
    }

    Ok(event)
}

fn notify_event_created(
    connections: &sqlite::Connections,
    notify: &dyn NotificationGateway,
    event: &Event,
) -> Result<()> {
    if let Some(ref location) = event.location {
        let email_addresses = {
            let conn = connections.shared()?;
            usecases::email_addresses_by_coordinate(&conn, location.pos)?
        };
        notify.event_created(&email_addresses, event);
    }
    Ok(())
}
