use ofdb_core::gateways::notify::NotificationGateway;

use super::*;
use crate::infrastructure::cfg::Cfg;

#[allow(clippy::too_many_arguments)]
pub fn update_place(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    notify: &dyn NotificationGateway,
    id: Id,
    update_place: usecases::UpdatePlace,
    created_by_email: Option<&str>,
    created_by_org: Option<&Organization>,
    cfg: &Cfg,
) -> Result<Place> {
    // Update existing entry
    let (place, ratings) = {
        let connection = connections.exclusive()?;
        connection.transaction(|| {
            match usecases::prepare_updated_place(
                &connection,
                id,
                update_place,
                created_by_email,
                created_by_org,
                &cfg.accepted_licenses,
            ) {
                Ok(storable) => {
                    let (place, ratings) = usecases::store_updated_place(&connection, storable)
                        .map_err(|err| {
                            warn!("Failed to store updated place: {}", err);
                            TransactionError::RollbackTransaction
                        })?;
                    Ok((place, ratings))
                }
                Err(err) => Err(TransactionError::Usecase(err)),
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
