use super::*;

use diesel::connection::Connection;

pub fn archive_events(connections: &sqlite::Connections, uids: &[&str]) -> Result<()> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_events(&*connection, uids).map_err(|err| {
                warn!("Failed to archive {} events: {}", uids.len(), err);
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
