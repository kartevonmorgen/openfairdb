use super::*;

use diesel::connection::Connection;

fn exec_archive_places(
    connections: &sqlite::Connections,
    ids: &[&str],
    archived_by_email: &str,
) -> Result<usize> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::review_places(&*connection, ids, ReviewStatus::Archived, archived_by_email)
                .map_err(|err| {
                    warn!("Failed to archive {} places: {}", ids.len(), err);
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

fn post_archive_places(indexer: &mut dyn PlaceIndexer, ids: &[&str]) -> Result<()> {
    // Remove archived entries from search index
    // TODO: Move to a separate task/thread that doesn't delay this request
    for id in ids {
        if let Err(err) = usecases::unindex_place(indexer, id) {
            error!(
                "Failed to remove archived place {} from search index: {}",
                id, err
            );
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!(
            "Failed to finish updating the search index after archiving places: {}",
            err
        );
    }
    Ok(())
}

pub fn archive_places(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    ids: &[&str],
    archived_by_email: &str,
) -> Result<usize> {
    let count = exec_archive_places(connections, ids, archived_by_email)?;
    post_archive_places(indexer, ids)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn archive_places(
        fixture: &EnvFixture,
        ids: &[&str],
        archived_by_email: &str,
    ) -> super::Result<usize> {
        super::archive_places(
            &fixture.db_connections,
            &mut *fixture.search_engine.borrow_mut(),
            ids,
            archived_by_email,
        )
    }

    #[test]
    fn should_archive_multiple_places_only_once() {
        let fixture = EnvFixture::new();

        fixture.create_user(
            usecases::NewUser {
                email: "test@example.com".into(),
                password: "test123".into(),
            },
            None,
        );

        let place_ids = vec![
            fixture.create_place(0.into(), None),
            fixture.create_place(1.into(), None),
            fixture.create_place(2.into(), None),
        ];
        let entry_tags = vec![
            fixture
                .try_get_place(&place_ids[0])
                .unwrap()
                .0
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
            fixture
                .try_get_place(&place_ids[1])
                .unwrap()
                .0
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
            fixture
                .try_get_place(&place_ids[2])
                .unwrap()
                .0
                .tags
                .into_iter()
                .take(1)
                .next()
                .unwrap(),
        ];

        assert!(fixture.place_exists(&place_ids[0]));
        assert_eq!(
            place_ids[0],
            fixture.query_places_by_tag(&entry_tags[0])[0].id
        );
        assert!(fixture.place_exists(&place_ids[1]));
        assert_eq!(
            place_ids[1],
            fixture.query_places_by_tag(&entry_tags[1])[0].id
        );
        assert!(fixture.place_exists(&place_ids[2]));
        assert_eq!(
            place_ids[2],
            fixture.query_places_by_tag(&entry_tags[2])[0].id
        );

        assert_eq!(
            2,
            archive_places(
                &fixture,
                &[&*place_ids[0], &*place_ids[2]],
                "test@example.com"
            )
            .unwrap()
        );

        // Entries 0 and 2 disappeared
        assert!(!fixture.place_exists(&place_ids[0]));
        assert!(fixture.query_places_by_tag(&entry_tags[0]).is_empty());
        assert!(fixture.place_exists(&place_ids[1]));
        assert_eq!(
            place_ids[1],
            fixture.query_places_by_tag(&entry_tags[1])[0].id
        );
        assert!(!fixture.place_exists(&place_ids[2]));
        assert!(fixture.query_places_by_tag(&entry_tags[2]).is_empty());

        assert_eq!(
            0,
            archive_places(
                &fixture,
                &[&*place_ids[0], &*place_ids[2]],
                "test@example.com",
            )
            .unwrap()
        );

        // No changes, i.e. entry 1 still exists in both database and index
        assert!(!fixture.place_exists(&place_ids[0]));
        assert!(fixture.query_places_by_tag(&entry_tags[0]).is_empty());
        assert!(fixture.place_exists(&place_ids[1]));
        assert_eq!(
            place_ids[1],
            fixture.query_places_by_tag(&entry_tags[1])[0].id
        );
        assert!(!fixture.place_exists(&place_ids[2]));
        assert!(fixture.query_places_by_tag(&entry_tags[2]).is_empty());

        // Archive all (remaining) places
        assert_eq!(
            1,
            archive_places(
                &fixture,
                &place_ids.iter().map(String::as_str).collect::<Vec<_>>(),
                "test@example.com",
            )
            .unwrap()
        );

        assert!(!fixture.place_exists(&place_ids[0]));
        assert!(fixture.query_places_by_tag(&entry_tags[0]).is_empty());
        assert!(!fixture.place_exists(&place_ids[1]));
        assert!(fixture.query_places_by_tag(&entry_tags[1]).is_empty());
        assert!(!fixture.place_exists(&place_ids[2]));
        assert!(fixture.query_places_by_tag(&entry_tags[2]).is_empty());
    }

    #[test]
    fn should_archive_places_with_ratings_and_comments() {
        let fixture = EnvFixture::new();

        fixture.create_user(
            usecases::NewUser {
                email: "test@example.com".into(),
                password: "test123".into(),
            },
            None,
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

        for place_id in &place_ids {
            assert!(fixture.place_exists(place_id));
        }
        for (rating_id, comment_id) in &rating_comment_ids {
            assert!(fixture.rating_exists(rating_id));
            assert!(fixture.comment_exists(comment_id));
        }

        assert_eq!(
            1,
            archive_places(&fixture, &[&*place_ids[0]], "test@example.com").unwrap()
        );

        assert!(!fixture.place_exists(&place_ids[0]));
        assert!(fixture.place_exists(&place_ids[1]));

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
