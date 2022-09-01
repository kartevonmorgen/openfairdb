use std::result;

use ofdb_core::gateways::notify::NotificationGateway;
use ofdb_db_sqlite::DbReadWrite;

use super::*;
use usecases::{Error, NewEvent, NewEventMode};

pub fn create_event(
    connections: &sqlite::Connections,
    indexer: &mut dyn EventIndexer,
    notify: &dyn NotificationGateway,
    token: Option<&str>,
    new_event: NewEvent,
) -> Result<Event> {
    let event = create_and_add_new_event(connections.exclusive()?, token, new_event)?;

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

fn create_and_add_new_event(
    mut connection: DbReadWrite<'_>,
    token: Option<&str>,
    new_event: NewEvent,
) -> result::Result<Event, Error> {
    connection.transaction(|conn| {
        let result = usecases::import_new_event(conn, token, new_event, NewEventMode::Create);
        match result {
            Ok(storable) => {
                let event = usecases::store_created_event(conn, storable).map_err(|err| {
                    warn!("Failed to store newly created event: {}", err);
                    err
                })?;
                Ok(event)
            }
            Err(err) => Err(err),
        }
    })
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
