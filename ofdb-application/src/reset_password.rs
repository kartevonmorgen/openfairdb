use ofdb_core::gateways::notify::NotificationGateway;

use super::*;

fn refresh_user_token(connections: &sqlite::Connections, user: &User) -> Result<EmailNonce> {
    Ok(connections
        .exclusive()?
        .transaction(|conn| usecases::refresh_user_token(conn, user.email.to_owned()))?)
}

pub fn reset_password_request(
    connections: &sqlite::Connections,
    notify: &dyn NotificationGateway,
    email: &str,
) -> Result<EmailNonce> {
    // The user is loaded before the following transaction that
    // requires exclusive access to the database connection for
    // writing.
    let user = connections.shared()?.get_user_by_email(email)?;
    let email_nonce = refresh_user_token(connections, &user)?;
    notify.user_reset_password_requested(&email_nonce);
    Ok(email_nonce)
}

pub fn reset_password_with_email_nonce(
    connections: &sqlite::Connections,
    email_nonce: EmailNonce,
    new_password: Password,
) -> Result<()> {
    // The token should be consumed only once, even if the
    // following transaction for updating the user fails!
    let token = connections.exclusive()?.transaction(|conn| {
        usecases::consume_user_token(conn, &email_nonce).map_err(|err| {
            log::warn!(
                "Missing or invalid token to reset password for user '{}': {}",
                email_nonce.email,
                err
            );
            err
        })
    })?;

    // The consumed nonce must match the request parameters
    debug_assert!(token.email_nonce == email_nonce);

    // Verify and update the user entity
    connections.exclusive()?.transaction(|conn| {
        usecases::confirm_email_and_reset_password(conn, &token.email_nonce.email, new_password)
            .map_err(|err| {
                warn!(
                    "Failed to verify e-mail ({}) and reset password: {}",
                    token.email_nonce.email, err
                );
                err
            })
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn reset_password_request(fixture: &BackendFixture, email: &str) -> super::Result<EmailNonce> {
        super::reset_password_request(&fixture.db_connections, &fixture.notify, email)
    }

    fn reset_password_with_email_nonce(
        fixture: &BackendFixture,
        email_nonce: EmailNonce,
        new_password: Password,
    ) -> super::Result<()> {
        super::reset_password_with_email_nonce(&fixture.db_connections, email_nonce, new_password)
    }

    #[test]
    fn should_reset_password() {
        let fixture = BackendFixture::new();

        // User 1
        let email1 = "user1@some.org";
        let credentials1 = usecases::Credentials {
            email: email1,
            password: "new pass1",
        };
        fixture.create_user(
            usecases::NewUser {
                email: email1.to_string(),
                password: "old pass1".to_string(),
            },
            None,
        );

        // User 2
        let email2 = "user2@some.org";
        let credentials2 = usecases::Credentials {
            email: email2,
            password: "new pass2",
        };
        fixture.create_user(
            usecases::NewUser {
                email: email2.to_string(),
                password: "old pass2".to_string(),
            },
            None,
        );

        // Verify that password is invalid for both users
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_err());
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_err());

        // Request and reset password for user 1 (by email)
        let email_nonce1 = reset_password_request(&fixture, email1).unwrap();
        assert_eq!(email1, email_nonce1.email);

        // Request and reset password for user 2
        let email_nonce2 = reset_password_request(&fixture, email2).unwrap();
        assert_eq!(email2, email_nonce2.email);

        // Reset the password of user 1
        assert!(reset_password_with_email_nonce(
            &fixture,
            email_nonce1.clone(),
            credentials1.password.parse::<Password>().unwrap()
        )
        .is_ok());
        // Verify that a 2nd attempt to reset the password with the same token fails
        assert!(reset_password_with_email_nonce(
            &fixture,
            email_nonce1,
            credentials1.password.parse::<Password>().unwrap()
        )
        .is_err());

        // Check that user 1 is able to login with the new password
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_ok());
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_err());

        assert!(reset_password_with_email_nonce(
            &fixture,
            email_nonce2.clone(),
            credentials2.password.parse::<Password>().unwrap()
        )
        .is_ok());
        // Verify that a 2nd attempt to reset the password with the same token fails
        assert!(reset_password_with_email_nonce(
            &fixture,
            email_nonce2,
            credentials2.password.parse::<Password>().unwrap()
        )
        .is_err());

        // Check that both users are able to login with their new passwords
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials1
        )
        .is_ok());
        debug_assert!(usecases::login_with_email(
            &fixture.db_connections.shared().unwrap(),
            &credentials2
        )
        .is_ok());
    }
}
