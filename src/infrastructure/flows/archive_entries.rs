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

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn archive_entries(fixture: &EnvFixture, ids: &[&str]) -> super::Result<()> {
        super::archive_entries(
            &fixture.db_connections,
            &mut *fixture.search_engine.borrow_mut(),
            ids,
        )
    }

    #[test]
    fn should_archive_multiple_entries_only_once() {
        let fixture = EnvFixture::new();
        let entry_ids = vec![
            fixture.add_entry(0.into()),
            fixture.add_entry(1.into()),
            fixture.add_entry(2.into()),
        ];
        let entry_tags = vec![
            fixture
                .try_get_entry(&entry_ids[0])
                .unwrap()
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
            fixture
                .try_get_entry(&entry_ids[1])
                .unwrap()
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
            fixture
                .try_get_entry(&entry_ids[2])
                .unwrap()
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
        ];

        assert!(fixture.entry_exists(&entry_ids[0]));
        assert_eq!(
            entry_ids[0],
            fixture.query_entries_by_tag(&entry_tags[0])[0].id
        );
        assert!(fixture.entry_exists(&entry_ids[1]));
        assert_eq!(
            entry_ids[1],
            fixture.query_entries_by_tag(&entry_tags[1])[0].id
        );
        assert!(fixture.entry_exists(&entry_ids[2]));
        assert_eq!(
            entry_ids[2],
            fixture.query_entries_by_tag(&entry_tags[2])[0].id
        );

        assert!(archive_entries(&fixture, &vec![&*entry_ids[0], &*entry_ids[2]]).is_ok());

        // Entries 0 and 2 disappeared
        assert!(!fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.query_entries_by_tag(&entry_tags[0]).is_empty());
        assert!(fixture.entry_exists(&entry_ids[1]));
        assert_eq!(
            entry_ids[1],
            fixture.query_entries_by_tag(&entry_tags[1])[0].id
        );
        assert!(!fixture.entry_exists(&entry_ids[2]));
        assert!(fixture.query_entries_by_tag(&entry_tags[2]).is_empty());

        assert!(archive_entries(&fixture, &vec![&*entry_ids[1], &*entry_ids[2]]).is_err());

        // No changes, i.e.entry 1 still exists in both database and index
        assert!(!fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.query_entries_by_tag(&entry_tags[0]).is_empty());
        assert!(fixture.entry_exists(&entry_ids[1]));
        assert_eq!(
            entry_ids[1],
            fixture.query_entries_by_tag(&entry_tags[1])[0].id
        );
        assert!(!fixture.entry_exists(&entry_ids[2]));
        assert!(fixture.query_entries_by_tag(&entry_tags[2]).is_empty());
    }
}
