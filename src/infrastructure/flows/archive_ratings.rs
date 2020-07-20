use super::*;

use diesel::connection::Connection;

pub fn exec_archive_ratings(
    connections: &sqlite::Connections,
    account_email: &str,
    ids: &[&str],
) -> Result<usize> {
    //TODO: check if user is allowed to archive the ratings
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_ratings(&*connection, account_email, ids).map_err(|err| {
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
    indexer: &mut dyn PlaceIndexer,
    ids: &[&str],
) -> Result<()> {
    let connection = connections.shared()?;
    let place_ids = connection.load_place_ids_of_ratings(ids)?;
    for place_id in place_ids {
        let (place, status) = match connection.get_place(&place_id) {
            Ok(place) => place,
            Err(err) => {
                error!(
                    "Failed to load place {} for reindexing after archiving ratings: {}",
                    place_id, err
                );
                // Skip place
                continue;
            }
        };
        let ratings = match connection.load_ratings_of_place(place.id.as_ref()) {
            Ok(ratings) => ratings,
            Err(err) => {
                error!(
                    "Failed to load ratings for place {} for reindexing after archiving ratings: {}",
                    place.id, err
                );
                // Skip place
                continue;
            }
        };
        if let Err(err) = usecases::reindex_place(indexer, &place, status, &ratings) {
            error!(
                "Failed to reindex place {} after archiving ratings: {}",
                place.id, err
            );
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!(
            "Failed to finish updating the search index after archiving ratings: {}",
            err
        );
    }
    Ok(())
}

pub fn archive_ratings(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    account_email: &str,
    ids: &[&str],
) -> Result<usize> {
    let count = exec_archive_ratings(connections, account_email, ids)?;
    post_archive_ratings(connections, indexer, ids)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn archive_ratings(fixture: &BackendFixture, ids: &[&str]) -> super::Result<usize> {
        super::archive_ratings(
            &fixture.db_connections,
            &mut *fixture.search_engine.borrow_mut(),
            "scout@foo.tld",
            ids,
        )
    }

    #[test]
    fn should_archive_multiple_ratings_only_once() {
        let fixture = BackendFixture::new();

        fixture.create_user(
            usecases::NewUser {
                email: "scout@foo.tld".into(),
                password: "123456".into(),
            },
            Some(Role::Scout),
        );

        let place_ids = vec![
            fixture.create_place(0.into(), None),
            fixture.create_place(1.into(), None),
        ];
        let rating_comment_ids = vec![
            fixture.create_rating(new_entry_rating(
                0,
                &place_ids[0],
                RatingContext::Diversity,
                RatingValue::new(-1),
            )),
            fixture.create_rating(new_entry_rating(
                1,
                &place_ids[0],
                RatingContext::Fairness,
                RatingValue::new(0),
            )),
            fixture.create_rating(new_entry_rating(
                2,
                &place_ids[1],
                RatingContext::Transparency,
                RatingValue::new(1),
            )),
            fixture.create_rating(new_entry_rating(
                3,
                &place_ids[1],
                RatingContext::Renewable,
                RatingValue::new(2),
            )),
        ];

        assert!(fixture.place_exists(&place_ids[0]));
        assert!(fixture.place_exists(&place_ids[1]));

        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Archive ratings 1 and 2
        assert_eq!(
            2,
            archive_ratings(
                &fixture,
                &[&*rating_comment_ids[1].0, &*rating_comment_ids[2].0]
            )
            .unwrap()
        );

        // Entries still exist
        assert!(fixture.place_exists(&place_ids[0]));
        assert!(fixture.place_exists(&place_ids[1]));

        // Ratings 1 and 2 disappeared
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        // Comments for ratings 1 and 2 disappeared
        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Try to archive ratings 0 and 1 (already archived)
        assert_eq!(
            1,
            archive_ratings(
                &fixture,
                &[&*rating_comment_ids[0].0, &*rating_comment_ids[1].0],
            )
            .unwrap()
        );

        // Archive all (remaining) ratings
        assert_eq!(
            1,
            archive_ratings(
                &fixture,
                &rating_comment_ids
                    .iter()
                    .map(|(r, _c)| r.as_str())
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );

        // Entries still exist
        assert!(fixture.place_exists(&place_ids[0]));
        assert!(fixture.place_exists(&place_ids[1]));

        // All ratings disappeared
        assert!(!fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(!fixture.rating_exists(&rating_comment_ids[3].0));

        // All comments disappeared
        assert!(!fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[3].1));
    }
}
