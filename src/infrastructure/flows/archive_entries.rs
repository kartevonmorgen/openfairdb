use super::*;

use diesel::connection::Connection;

pub fn exec_archive_entries(connections: &sqlite::Connections, ids: &[&str]) -> Result<()> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_entries(&*connection, ids).map_err(|err| {
                warn!("Failed to archive {} entries: {}", ids.len(), err);
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

pub fn post_archive_entries(indexer: &mut dyn EntryIndexer, ids: &[&str]) -> Result<()> {
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

pub fn archive_entries(
    connections: &sqlite::Connections,
    indexer: &mut dyn EntryIndexer,
    ids: &[&str],
) -> Result<()> {
    exec_archive_entries(connections, ids)?;
    post_archive_entries(indexer, ids)?;
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
            fixture.create_entry(0.into()),
            fixture.create_entry(1.into()),
            fixture.create_entry(2.into()),
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

        assert!(archive_entries(&fixture, &[&*entry_ids[0], &*entry_ids[2]]).is_ok());

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

        assert_not_found(archive_entries(&fixture, &[&*entry_ids[1], &*entry_ids[2]]));

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

    #[test]
    fn should_archive_entries_with_ratings_and_comments() {
        let fixture = EnvFixture::new();
        let entry_ids = vec![
            fixture.create_entry(0.into()),
            fixture.create_entry(1.into()),
        ];

        let rating_comment_ids = vec![
            fixture.create_rating(new_entry_rating(
                0,
                &entry_ids[0],
                RatingContext::Diversity,
                RatingValue::new(-1),
            )),
            fixture.create_rating(new_entry_rating(
                1,
                &entry_ids[0],
                RatingContext::Fairness,
                RatingValue::new(0),
            )),
            fixture.create_rating(new_entry_rating(
                2,
                &entry_ids[1],
                RatingContext::Transparency,
                RatingValue::new(1),
            )),
            fixture.create_rating(new_entry_rating(
                3,
                &entry_ids[1],
                RatingContext::Renewable,
                RatingValue::new(2),
            )),
        ];

        for entry_id in &entry_ids {
            assert!(fixture.entry_exists(entry_id));
        }
        for (rating_id, comment_id) in &rating_comment_ids {
            assert!(fixture.rating_exists(rating_id));
            assert!(fixture.comment_exists(comment_id));
        }

        assert!(archive_entries(&fixture, &[&*entry_ids[0]]).is_ok());

        assert!(!fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.entry_exists(&entry_ids[1]));

        assert!(!fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        assert!(!fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));
    }
}
