use super::*;

use diesel::connection::Connection;

pub fn archive_comments(connections: &sqlite::Connections, ids: &[&str]) -> Result<()> {
    let connection = connections.exclusive()?;
    connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_comments(&*connection, ids).map_err(|err| {
                warn!("Failed to archive {} comments: {}", ids.len(), err);
                diesel::result::Error::RollbackTransaction
            })
        })
        .map_err(|err| RepoError::from(err))?;
    Ok(())
}
