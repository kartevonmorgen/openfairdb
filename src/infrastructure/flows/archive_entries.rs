use super::*;

use diesel::connection::Connection;

pub fn archive_entries(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    ids: &[&str],
) -> Result<()> {
    let connection = connections.exclusive()?;
    connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_entries(&*connection, ids).map_err(|err| {
                warn!("Failed to archive {} entries: {}", ids.len(), err);
                diesel::result::Error::RollbackTransaction
            })
        })
        .map_err(|err| RepoError::from(err))?;

    // Remove archived entries from search index
    // TODO: Move to a separate task/thread that doesn't delay this request
    for id in ids {
        if let Err(err) = usecases::unindex_entry(indexer, id) {
            error!(
                "Failed to remove archived entry {} from search index: {}",
                id, err
            );
        }
    }
    if let Err(err) = indexer.flush() {
        error!(
            "Failed to finish updating the search index after archiving entries: {}",
            err
        );
    }

    Ok(())
}
