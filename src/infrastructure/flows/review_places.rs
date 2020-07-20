use super::*;

use diesel::connection::Connection;

fn exec_review_places(
    connections: &sqlite::Connections,
    ids: &[&str],
    review: usecases::Review,
) -> Result<usize> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::review_places(&*connection, ids, review).map_err(|err| {
                warn!("Failed to review {} places: {}", ids.len(), err);
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

fn post_review_places(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    ids: &[&str],
) -> Result<()> {
    let db = connections.shared()?;
    let places_with_status = db.get_places(ids)?;
    for (place, status) in places_with_status {
        let ratings = match db.load_ratings_of_place(place.id.as_str()) {
            Ok(ratings) => ratings,
            Err(err) => {
                log::error!(
                    "Failed to load ratings of place {} after reviewing: {}",
                    place.id,
                    err
                );
                continue;
            }
        };
        if let Err(err) = usecases::reindex_place(indexer, &place, status, &ratings) {
            error!(
                "Failed to (re-)index place {} after reviewing: {}",
                place.id, err
            );
        }
    }
    if let Err(err) = indexer.flush_index() {
        error!(
            "Failed to flush search index after reviewing places: {}",
            err
        );
    }
    Ok(())
}

pub fn review_places(
    connections: &sqlite::Connections,
    indexer: &mut dyn PlaceIndexer,
    ids: &[&str],
    review: usecases::Review,
) -> Result<usize> {
    let count = exec_review_places(connections, ids, review)?;
    // TODO: Move post processing to a separate task/thread that doesn't delay this request?
    post_review_places(connections, indexer, ids)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn review_places(
        fixture: &BackendFixture,
        ids: &[&str],
        review: usecases::Review,
    ) -> super::Result<usize> {
        super::review_places(
            &fixture.db_connections,
            &mut *fixture.search_engine.borrow_mut(),
            ids,
            review,
        )
    }

    fn archived_by(reviewer_email: &str) -> usecases::Review {
        usecases::Review {
            context: None,
            reviewer_email: reviewer_email.into(),
            status: ReviewStatus::Archived,
            comment: Some("Archived".into()),
        }
    }

    #[test]
    fn should_archive_multiple_places_only_once() {
        let fixture = BackendFixture::new();

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
            review_places(
                &fixture,
                &[&*place_ids[0], &*place_ids[2]],
                archived_by("test@example.com"),
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
            review_places(
                &fixture,
                &[&*place_ids[0], &*place_ids[2]],
                archived_by("test@example.com"),
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
            review_places(
                &fixture,
                &place_ids.iter().map(String::as_str).collect::<Vec<_>>(),
                archived_by("test@example.com"),
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
    fn should_archive_places_and_leaving_ratings_and_comments_unchanged() {
        let fixture = BackendFixture::new();

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
            review_places(&fixture, &[&*place_ids[0]], archived_by("test@example.com")).unwrap()
        );

        assert!(!fixture.place_exists(&place_ids[0]));
        assert!(fixture.place_exists(&place_ids[1]));

        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));
    }
}
