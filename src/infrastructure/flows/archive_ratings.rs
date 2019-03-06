use super::*;

use diesel::connection::Connection;

pub fn exec_archive_ratings(connections: &sqlite::Connections, ids: &[&str]) -> Result<()> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_ratings(&*connection, ids).map_err(|err| {
                warn!("Failed to archive {} ratings: {}", ids.len(), err);
                repo_err = Some(err);
                diesel::result::Error::RollbackTransaction
            })
        })
        .map_err(|err| {
            if let Some(repo_err) = repo_err {
                repo_err
            } else {
                RepoError::from(err).into()
            }
        })?)
}

pub fn post_archive_ratings(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    ids: &[&str],
) -> Result<()> {
    let connection = connections.shared()?;
    let entry_ids = connection.load_entry_ids_of_ratings(ids)?;
    for entry_id in entry_ids {
        let entry = match connection.get_entry(&entry_id) {
            Ok(entry) => entry,
            Err(err) => {
                error!(
                    "Failed to load entry {} for reindexing after archiving ratings: {}",
                    entry_id, err
                );
                // Skip entry
                continue;
            }
        };
        let ratings = match connection.load_ratings_of_entry(&entry.id) {
            Ok(ratings) => ratings,
            Err(err) => {
                error!(
                    "Failed to load ratings for entry {} for reindexing after archiving ratings: {}",
                    entry.id, err
                );
                // Skip entry
                continue;
            }
        };
        if let Err(err) = usecases::index_entry(indexer, &entry, &ratings) {
            error!(
                "Failed to reindex entry {} after archiving ratings: {}",
                entry.id, err
            );
        }
    }
    if let Err(err) = indexer.flush() {
        error!(
            "Failed to finish updating the search index after archiving ratings: {}",
            err
        );
    }
    Ok(())
}

pub fn archive_ratings(
    connections: &sqlite::Connections,
    indexer: &mut EntryIndexer,
    ids: &[&str],
) -> Result<()> {
    exec_archive_ratings(connections, ids)?;
    post_archive_ratings(connections, indexer, ids)?;
    Ok(())
}
