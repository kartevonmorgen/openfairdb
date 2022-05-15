use diesel::connection::Connection;

use super::*;

fn exec_archive_events(
    connections: &sqlite::Connections,
    ids: &[&str],
    _archived_by_email: &str,
) -> Result<usize> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_events(&*connection, ids).map_err(|err| {
                warn!("Failed to archive {} events: {}", ids.len(), err);
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

fn post_archive_events(indexer: &mut dyn EventIndexer, ids: &[&str]) {
    // Remove archived events from search index
    for id in ids {
        if let Err(err) = usecases::unindex_event(indexer, &Id::from(*id)) {
            error!(
                "Failed to remove archived event {} from search index: {}",
                id, err
            );
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!(
            "Failed to finish updating the search index after archiving events: {}",
            err
        );
    }
}

pub fn archive_events(
    connections: &sqlite::Connections,
    indexer: &mut dyn EventIndexer,
    ids: &[&str],
    archived_by_email: &str,
) -> Result<usize> {
    let count = exec_archive_events(connections, ids, archived_by_email)?;
    // TODO: Move post processing to a separate task/thread that doesn't delay this
    // request
    post_archive_events(indexer, ids);
    Ok(count)
}
