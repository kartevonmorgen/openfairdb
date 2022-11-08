use super::*;

use ofdb_core::gateways::notify::NotificationGateway;
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub fn update_place(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    notify: &dyn NotificationGateway,
    id: Id,
    update_place: usecases::UpdatePlace,
    created_by_email: Option<&EmailAddress>,
    created_by_org: Option<&Organization>,
    accepted_licenses: &HashSet<String>,
) -> Result<Place> {
    // Update existing entry
    let (place, ratings) = {
        connections.exclusive()?.transaction(|conn| {
            match usecases::prepare_updated_place(
                conn,
                id,
                update_place,
                created_by_email,
                created_by_org,
                accepted_licenses,
            ) {
                Ok(storable) => {
                    let (place, ratings) =
                        usecases::store_updated_place(conn, storable).map_err(|err| {
                            warn!("Failed to store updated place: {}", err);
                            err
                        })?;
                    Ok((place, ratings))
                }
                Err(err) => Err(err),
            }
        })
    }?;

    // Reindex updated place
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::reindex_place(indexer, &place, ReviewStatus::Created, &ratings)
        .and_then(|_| indexer.flush_index())
    {
        error!("Failed to reindex updated place {}: {}", place.id, err);
    }

    // Send subscription e-mails
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = notify_place_updated(connections, notify, &place) {
        error!(
            "Failed to send notifications for updated place {}: {}",
            place.id, err
        );
    }

    Ok(place)
}

fn notify_place_updated(
    connections: &sqlite::Connections,
    notify: &dyn NotificationGateway,
    place: &Place,
) -> Result<()> {
    let (email_addresses, all_categories) = {
        let connection = connections.shared()?;
        let email_addresses =
            usecases::email_addresses_by_coordinate(&connection, place.location.pos)?;
        let all_categories = connection.all_categories()?;
        (email_addresses, all_categories)
    };
    notify.place_updated(&email_addresses, place, all_categories);
    Ok(())
}
