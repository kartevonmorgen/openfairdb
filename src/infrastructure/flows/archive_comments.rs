use super::*;
use diesel::connection::Connection;

pub fn archive_comments(
    connections: &sqlite::Connections,
    account_email: &str,
    ids: &[&str],
) -> Result<()> {
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
    ) -> super::Result<()> {
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

        assert!(fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.entry_exists(&entry_ids[1]));
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));

        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Archive comments 1 and 2
        assert!(archive_comments(
            &fixture,
            "scout@foo.tld",
            &[&*rating_comment_ids[1].1, &*rating_comment_ids[2].1]
        )
        .is_ok());

        // Entries and ratings still exist
        assert!(fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.entry_exists(&entry_ids[1]));
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
        assert_not_found(archive_comments(
            &fixture,
            "scout@foo.tld",
            &[&*rating_comment_ids[0].1, &*rating_comment_ids[1].1],
        ));

        // No changes due to rollback
        assert!(fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.entry_exists(&entry_ids[1]));
        assert!(fixture.rating_exists(&rating_comment_ids[0].0));
        assert!(fixture.rating_exists(&rating_comment_ids[1].0));
        assert!(fixture.rating_exists(&rating_comment_ids[2].0));
        assert!(fixture.rating_exists(&rating_comment_ids[3].0));
        assert!(fixture.comment_exists(&rating_comment_ids[0].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[1].1));
        assert!(!fixture.comment_exists(&rating_comment_ids[2].1));
        assert!(fixture.comment_exists(&rating_comment_ids[3].1));

        // Archive remaining comments
        assert!(archive_comments(
            &fixture,
            "scout@foo.tld",
            &[&*rating_comment_ids[0].1, &*rating_comment_ids[3].1]
        )
        .is_ok());

        // Entries and ratings still exist
        assert!(fixture.entry_exists(&entry_ids[0]));
        assert!(fixture.entry_exists(&entry_ids[1]));
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
