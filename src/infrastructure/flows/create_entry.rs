use super::*;

use crate::core::error::RepoError;

use diesel::Connection;

pub fn create_entry(
    connections: &sqlite::Connections,
    indexer: &mut dyn EntryIndexer,
    new_entry: usecases::NewEntry,
) -> Result<String> {
    // Create and add new entry
    let (entry, ratings) = {
        let connection = connections.exclusive()?;
        let mut prepare_err = None;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                match usecases::prepare_new_entry(&*connection, new_entry) {
                    Ok(storable) => {
                        let (entry, ratings) = usecases::store_new_entry(&*connection, storable)
                            .map_err(|err| {
                                warn!("Failed to store newly created entry: {}", err);
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

    // Index newly added entry
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = usecases::index_entry(indexer, &entry, &ratings).and_then(|_| indexer.flush())
    {
        error!("Failed to index newly added entry {}: {}", entry.uid, err);
    }

    // Send subscription e-mails
    // TODO: Move to a separate task/thread that doesn't delay this request
    if let Err(err) = notify_entry_added(connections, &entry) {
        error!(
            "Failed to send notifications for newly added entry {}: {}",
            entry.uid, err
        );
    }

    Ok(entry.uid.into())
}

fn notify_entry_added(connections: &sqlite::Connections, entry: &Entry) -> Result<()> {
    let (email_addresses, all_categories) = {
        let connection = connections.shared()?;
        let email_addresses =
            usecases::email_addresses_by_coordinate(&*connection, entry.location.pos)?;
        let all_categories = connection.all_categories()?;
        (email_addresses, all_categories)
    };
    notify::entry_added(&email_addresses, &entry, all_categories);
    Ok(())
}
