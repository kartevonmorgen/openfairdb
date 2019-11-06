use super::*;
use diesel::connection::Connection;

pub fn archive_comments(
    connections: &sqlite::Connections,
    account_email: &str,
    ids: &[&str],
) -> Result<usize> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::archive_comments(&*connection, account_email, ids).map_err(|err| {
                warn!("Failed to archive {} comments: {}", ids.len(), err);
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

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn archive_comments(
        fixture: &EnvFixture,
        account_email: &str,
        ids: &[&str],
    ) -> super::Result<usize> {
        super::archive_comments(&fixture.db_connections, account_email, ids)
    }

    #[test]
    fn should_archive_multiple_comments_only_once() {
        let fixture = EnvFixture::new();

        fixture.create_user(
            usecases::NewUser {
                email: "scout@foo.tld".into(),
                password: "123456".into(),
            },
            Some(Role::Scout),
        );

        let place_uids = vec![
            fixture.create_place(0.into(), None),
            fixture.create_place(1.into(), None),
        ];
        let rating_comment_ids = vec![
            fixture.create_rating(new_entry_rating(
                0,
                &place_uids[0],
                RatingContext::Diversity,
                RatingValue::new(-1),
            )),
            fixture.create_rating(new_entry_rating(
                1,
                &place_uids[0],
                RatingContext::Fairness,
                RatingValue::new(0),
            )),
            fixture.create_rating(new_entry_rating(
                2,
                &place_uids[1],
                RatingContext::Transparency,
                RatingValue::new(1),
            )),
            fixture.create_rating(new_entry_rating(
                3,
                &place_uids[1],
                RatingContext::Renewable,
                RatingValue::new(2),
            )),
        ];

        assert!(fixture.place_exists(&place_uids[0]));
        assert!(fixture.place_exists(&place_uids[1]));
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Archive comments 1 and 2
        assert_eq!(
            2,
            archive_comments(
                &fixture,
                "scout@foo.tld",
                &[&*rating_comment_ids[1].1, &*rating_comment_ids[2].1]
            )
            .unwrap()
        );

        // Entries and ratings still exist
        assert!(fixture.place_exists(&place_uids[0]));
        assert!(fixture.place_exists(&place_uids[1]));
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        // Comments 1 and 2 disappeared
        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Try to archive comments 0 and 1 (already archived)
        assert_eq!(
            1,
            archive_comments(
                &fixture,
                "scout@foo.tld",
                &[&*rating_comment_ids[0].1, &*rating_comment_ids[1].1],
            )
            .unwrap()
        );

        // Archive all (remaining) comments
        assert_eq!(
            1,
            archive_comments(
                &fixture,
                "scout@foo.tld",
                &rating_comment_ids
                    .iter()
                    .map(|(_r, c)| c.as_str())
                    .collect::<Vec<_>>(),
            )
            .unwrap()
        );

        // Entries and ratings still exist
        assert!(fixture.place_exists(&place_uids[0]));
        assert!(fixture.place_exists(&place_uids[1]));
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        // All comments disappeared
        assert!(!fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[3].1));
    }
}
