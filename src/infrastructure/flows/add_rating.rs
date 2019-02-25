use super::*;

use diesel::connection::Connection;

pub fn add_rating(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    rate_entry: usecases::RateEntry,
) -> Result<Entry> {
    // Add new rating to existing entry
    let (entry, ratings) = {
        let connection = connections.exclusive()?;
        // TODO: Move creation & validation into transaction
        let storable = usecases::prepare_new_rating(&*connection, rate_entry)?;
        connection
            .transaction::<_, diesel::result::Error, _>(|| {
                let (entry, ratings) =
                    usecases::store_new_rating(&*connection, storable).map_err(|err| {
                        warn!("Failed to store new rating for entry: {}", err);
                        diesel::result::Error::RollbackTransaction
                    })?;
                Ok((entry, ratings))
            })
            .map_err(RepoError::from)
    }?;

    // Reindex entry after adding the new rating
    if let Err(err) = usecases::index_entry(indexer, &entry, &ratings).and_then(|_| indexer.flush())
    {
        error!(
            "Failed to reindex entry {} after adding a new rating: {}",
            entry.id, err
        );
    }

    Ok(entry)
}
