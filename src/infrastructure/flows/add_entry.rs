use super::*;

use crate::core::error::RepoError;

use diesel::Connection;

pub fn add_entry(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    new_entry: usecases::NewEntry,
) -> Result<Entry> {
    // Create and add new entry
    let (entry, ratings) = {
        let connection = connections.exclusive()?;
        // TODO: Move creation & validation into transaction
        let storable = usecases::prepare_new_entry(&*connection, new_entry)?;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                let (entry, ratings) =
                    usecases::store_new_entry(&*connection, storable).map_err(|err| {
                        warn!("Failed to store newly created entry: {}", err);
                        diesel::result::Error::RollbackTransaction
                    })?;
                Ok((entry, ratings))
            })
            .map_err(RepoError::from)
    }?;

    // Index newly added entry
    if let Err(err) = usecases::index_entry(indexer, &entry, &ratings).and_then(|_| indexer.flush())
    {
        error!("Failed to index newly added entry {}: {}", entry.id, err);
    }

    // Send subscription e-mails
    if let Err(err) = notify_entry_added(connections, &entry) {
        error!(
            "Failed to send notifications for newly added entry {}: {}",
            entry.id, err
        );
    }

    Ok(entry)
}

fn notify_entry_added(connections: &sqlite::Connections, entry: &Entry) -> Result<()> {
    let (email_addresses, all_categories) = {
        let connection = connections.shared()?;
        let email_addresses = usecases::email_addresses_by_coordinate(
            &*connection,
            entry.location.lat,
            entry.location.lng,
        )?;
        let all_categories = connection.all_categories()?;
        (email_addresses, all_categories)
    };
    notify::entry_added(&email_addresses, &entry, all_categories);
    Ok(())
}
