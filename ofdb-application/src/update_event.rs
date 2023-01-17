use ofdb_core::gateways::notify::{NotificationEvent, NotificationGateway};

use super::*;

pub fn update_event(
    connections: &sqlite::Connections,
    indexer: &mut dyn EventIndexer,
    notify: &dyn NotificationGateway,
    token: Option<&str>,
    id: Id,
    new_event: usecases::NewEvent,
) -> Result<Event> {
    // Create and add new event
    let event = {
        connections.exclusive()?.transaction(|conn| {
            match usecases::import_new_event(
                conn,
                token,
                new_event,
                usecases::NewEventMode::Update(id.as_str()),
            ) {
                Ok(storable) => {
                    let event = usecases::store_updated_event(conn, storable).map_err(|err| {
                        warn!("Failed to store updated event: {}", err);
                        err
                    })?;
                    Ok(event)
                }
                Err(err) => Err(err),
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
    if let Err(err) = notify_event_updated(connections, notify, &event) {
        error!(
            "Failed to send notifications for updated event {}: {}",
            event.id, err
        );
    }

    Ok(event)
}

fn notify_event_updated(
    connections: &sqlite::Connections,
    notify: &dyn NotificationGateway,
    event: &Event,
) -> Result<()> {
    if let Some(ref location) = event.location {
        let email_addresses = {
            let conn = connections.shared()?;
            usecases::email_addresses_by_coordinate(&conn, location.pos)?
        };
        let event = NotificationEvent::EventUpdated {
            event,
            email_addresses: &email_addresses,
        };
        notify.notify(event);
    }
    Ok(())
}
