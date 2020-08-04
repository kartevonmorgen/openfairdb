use super::*;
use diesel::connection::Connection;
use ofdb_core::NotificationGateway;

pub fn update_place(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    notify: &dyn NotificationGateway,
    id: Id,
    update_place: usecases::UpdatePlace,
    created_by_email: Option<&str>,
    created_by_org: Option<&Organization>,
) -> Result<Place> {
    // Update existing entry
    let (place, ratings) = {
        let connection = connections.exclusive()?;
        let mut prepare_err = None;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                match usecases::prepare_updated_place(
                    &*connection,
                    id,
                    update_place,
                    created_by_email,
                    created_by_org,
                ) {
                    Ok(storable) => {
                        let (place, ratings) =
                            usecases::store_updated_place(&*connection, storable).map_err(
                                |err| {
                                    warn!("Failed to store updated place: {}", err);
                                    diesel::result::Error::RollbackTransaction
                                },
                            )?;
                        Ok((place, ratings))
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
            usecases::email_addresses_by_coordinate(&*connection, place.location.pos)?;
        let all_categories = connection.all_categories()?;
        (email_addresses, all_categories)
    };
    notify.place_updated(&email_addresses, &place, all_categories);
    Ok(())
}
