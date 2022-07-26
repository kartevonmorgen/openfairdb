use ofdb_core::gateways::notify::NotificationGateway;

use super::*;
use crate::infrastructure::cfg::Cfg;

pub fn create_place(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    notify: &dyn NotificationGateway,
    new_place: usecases::NewPlace,
    created_by_email: Option<&str>,
    created_by_org: Option<&Organization>,
    cfg: &Cfg,
) -> Result<Place> {
    // Create and add new entry
    let (place, ratings) = {
        let connection = connections.exclusive()?;
        connection.transaction(|| {
            match usecases::prepare_new_place(
                &connection,
                new_place,
                created_by_email,
                created_by_org,
                &cfg.accepted_licenses,
            ) {
                Ok(storable) => {
                    let (place, ratings) = usecases::store_new_place(&connection, storable)
                        .map_err(|err| {
                            warn!("Failed to store newly created place: {}", err);
                            TransactionError::RollbackTransaction
                        })?;
                    Ok((place, ratings))
                }
                Err(err) => {
                    log::info!("Failed to prepare new place revision: {}", err);
                    Err(TransactionError::Usecase(err))
                }
            }
        })
    }?;

    // Index newly added place
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::reindex_place(indexer, &place, ReviewStatus::Created, &ratings)
        .and_then(|_| indexer.flush_index())
    {
        error!("Failed to index newly added place {}: {}", place.id, err);
    }

    // Send subscription e-mails
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = notify_place_added(connections, notify, &place) {
        error!(
            "Failed to send notifications for newly added place {}: {}",
            place.id, err
        );
    }

    Ok(place)
}

fn notify_place_added(
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
    notify.place_added(&email_addresses, place, all_categories);
    Ok(())
}
