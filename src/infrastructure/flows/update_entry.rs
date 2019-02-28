use super::*;

use diesel::connection::Connection;

pub fn update_entry(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    id: String,
    update_entry: usecases::UpdateEntry,
) -> Result<Entry> {
    // Update existing entry
    let (entry, ratings) = {
        let connection = connections.exclusive()?;
        let mut prepare_err = None;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                match usecases::prepare_updated_entry(&*connection, id, update_entry) {
                    Ok(storable) => {
                        let (entry, ratings) = usecases::store_updated_entry(&*connection, storable)
                            .map_err(|err| {
                                warn!("Failed to store updated entry: {}", err);
                                diesel::result::Error::RollbackTransaction
                            })?;
                        Ok((entry, ratings))
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

    // Reindex updated entry
    if let Err(err) = usecases::index_entry(indexer, &entry, &ratings).and_then(|_| indexer.flush())
    {
        error!("Failed to reindex updated entry {}: {}", entry.id, err);
    }

    // Send subscription e-mails
    if let Err(err) = notify_entry_updated(connections, &entry) {
        error!(
            "Failed to send notifications for updated entry {}: {}",
            entry.id, err
        );
    }

    Ok(entry)
}

fn notify_entry_updated(connections: &sqlite::Connections, entry: &Entry) -> Result<()> {
    let (email_addresses, all_categories) = {
        let connection = connections.shared()?;
        let email_addresses = usecases::email_addresses_by_coordinate(
            &*connection,
            &entry.location.pos,
        )?;
        let all_categories = connection.all_categories()?;
        (email_addresses, all_categories)
    };
    notify::entry_updated(&email_addresses, &entry, all_categories);
    Ok(())
}
