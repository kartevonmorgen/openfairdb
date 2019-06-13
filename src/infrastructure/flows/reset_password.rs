use super::*;

use crate::core::error::Error;

use diesel::connection::Connection;

// Try to load users either by e-mail (1st) or by username (2nd)
fn load_users_by_email_or_username<D: UserGateway>(
    db: &D,
    email_or_username: &str,
) -> Result<Vec<User>> {
    // Try with the username first (if available)
    match db.get_users_by_email(email_or_username) {
        Err(RepoError::NotFound) => {
            let user = db.get_user(email_or_username)?;
            Ok(vec![user])
        }
        res => {
            let users = res?;
            debug_assert!(!users.is_empty());
            if let Ok(user) = db.get_user(email_or_username) {
                for u in &users {
                    if u.username == user.username {
                        return Ok(vec![user]);
                    }
                }
                error!(
                    "Search results for users with e-mail or username '{}' are ambiguous!",
                    email_or_username
                );
                Err(RepoError::NotFound)?;
            }
            Ok(users)
        }
    }
}

fn refresh_email_token_credentials(
    connections: &sqlite::Connections,
    user: &User,
) -> Result<EmailTokenCredentials> {
    let mut rollback_err: Option<Error> = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::refresh_email_token_credentials(
                &*connection,
                user.username.to_owned(),
                user.email.to_owned(),
            )
            .map_err(|err| {
                rollback_err = Some(err);
                diesel::result::Error::RollbackTransaction
            })
        })
        .map_err(|err| rollback_err.unwrap_or_else(|| Error::from(RepoError::from(err))))?)
}

pub fn reset_password_request(
    connections: &sqlite::Connections,
    email_or_username: &str,
) -> Result<Vec<EmailToken>> {
    // The user is loaded before the following transaction that
    // requires exclusive access to the database connection for
    // writing.
    let users = load_users_by_email_or_username(&*connections.shared()?, email_or_username)?;
    if users.len() > 1 {
        warn!(
            "Requesting password reset for all ({}) users with e-mail address '{}'",
            users.len(),
            users[0].email
        );
    }

    let mut tokens = Vec::with_capacity(users.len());
    for user in &users {
        let credentials = refresh_email_token_credentials(&connections, &user)?;
        notify::user_reset_password_requested(&credentials.token);
        tokens.push(credentials.token);
    }

    Ok(tokens)
}

pub fn reset_password_with_email_token(
    connections: &sqlite::Connections,
    email_or_username: &str,
    token: EmailToken,
    new_password: Password,
) -> Result<()> {
    let connection = connections.exclusive()?;

    // The token should be consumed only once, even if the
    // following transaction for updating the user fails!
    let mut rollback_err: Option<Error> = None;
    let credentials = connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::consume_email_token_credentials(&*connection, email_or_username, &token)
                .map_err(|err| {
                    warn!(
                        "Missing or invalid token to reset password for user '{}': {}",
                        email_or_username, err
                    );
                    rollback_err = Some(err);
                    diesel::result::Error::RollbackTransaction
                })
        })
        .map_err(|err| rollback_err.unwrap_or_else(|| Error::from(RepoError::from(err))))?;

    // The consumed nonce must match the request parameters
    debug_assert!(credentials.token == token);

    // Verify and update the user entity
    let mut rollback_err: Option<Error> = None;
    connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::confirm_email_and_reset_password(
                &*connection,
                &credentials.username,
                &credentials.token.email,
                new_password,
            )
            .map_err(|err| {
                warn!(
                    "Failed to verify e-mail ({}) and reset password for user ({}): {}",
                    credentials.token.email, credentials.username, err
                );
                rollback_err = Some(err);
                diesel::result::Error::RollbackTransaction
            })
        })
        .map_err(|err| rollback_err.unwrap_or_else(|| Error::from(RepoError::from(err))))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn reset_password_request(
        fixture: &EnvFixture,
        email_or_username: &str,
    ) -> super::Result<Vec<EmailToken>> {
        super::reset_password_request(&fixture.db_connections, email_or_username)
    }

    fn reset_password_with_email_token(
        fixture: &EnvFixture,
        email_or_username: &str,
        token: EmailToken,
        new_password: Password,
    ) -> super::Result<()> {
        super::reset_password_with_email_token(
            &fixture.db_connections,
            email_or_username,
            token,
            new_password,
        )
    }

    #[test]
    fn should_reset_password() {
        let fixture = EnvFixture::new();

        // User 1
        let email1 = "user1@some.org";
        let username1 = fixture.create_user_from_email(email1);
        let credentials1 = usecases::Credentials {
            email: &email1,
            password: "new pass1",
        };

        // User 2
        let email2 = "user2@some.org";
        let username2 = fixture.create_user_from_email(email2);
        let credentials2 = usecases::Credentials {
            email: &email2,
            password: "new pass2",
        };

        // Verify that password is invalid for both users
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_err());
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_err());

        // Request and reset password for user 1 (by email)
        let tokens = reset_password_request(&fixture, email1).unwrap();
        assert_eq!(1, tokens.len());
        let token1 = tokens.into_iter().next().unwrap();
        assert_eq!(email1, token1.email);
        // Verify that we are not able to reset the password of another user with this token
        assert!(reset_password_with_email_token(
            &fixture,
            email2,
            token1.clone(),
            credentials2.password.parse::<Password>().unwrap()
        )
        .is_err());
        // Reset the password of user 1
        assert!(reset_password_with_email_token(
            &fixture,
            email1,
            token1.clone(),
            credentials1.password.parse::<Password>().unwrap()
        )
        .is_ok());
        // Verify that a 2nd attempt to reset the password with the same token fails
        assert!(reset_password_with_email_token(
            &fixture,
            email1,
            token1,
            credentials1.password.parse::<Password>().unwrap()
        )
        .is_err());

        // Check that user 1 is able to login with the new password
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_ok());
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_err());

        // Request and reset password for user 2 (by username)
        let tokens = reset_password_request(&fixture, &username2).unwrap();
        assert_eq!(1, tokens.len());
        let token2 = tokens.into_iter().next().unwrap();
        assert_eq!(email2, token2.email);
        // Verify that we are not able to reset the password of another user with this token
        assert!(reset_password_with_email_token(
            &fixture,
            &username1,
            token2.clone(),
            credentials1.password.parse::<Password>().unwrap()
        )
        .is_err());
        // Reset the password of user 2
        assert!(reset_password_with_email_token(
            &fixture,
            &username2,
            token2.clone(),
            credentials2.password.parse::<Password>().unwrap()
        )
        .is_ok());
        // Verify that a 2nd attempt to reset the password with the same token fails
        assert!(reset_password_with_email_token(
            &fixture,
            &username2,
            token2,
            credentials2.password.parse::<Password>().unwrap()
        )
        .is_err());

        // Check that both users are able to login with their new passwords
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_ok());
        debug_assert!(usecases::login_with_email(
            &*fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_ok());
    }
}
