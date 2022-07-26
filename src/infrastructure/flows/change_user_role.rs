use super::*;

pub fn change_user_role(
    connections: &sqlite::Connections,
    account_email: &str,
    user_email: &str,
    role: Role,
) -> Result<()> {
    let connection = connections.exclusive()?;
    Ok(connection.transaction(|| {
        usecases::change_user_role(&connection.inner(), account_email, user_email, role).map_err(
            |err| {
                log::warn!("Failed to change role for email {}: {}", user_email, err);
                err
            },
        )
    })?)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;

    fn change_user_role(
        fixture: &BackendFixture,
        account_email: &str,
        user_email: &str,
        role: Role,
    ) -> super::Result<()> {
        super::change_user_role(&fixture.db_connections, account_email, user_email, role)
    }

    #[test]
    fn should_change_the_role_to_scout_if_its_done_by_an_admin() {
        let fixture = BackendFixture::new();
        fixture.create_user(
            usecases::NewUser {
                email: "user@bar.tld".into(),
                password: "123456".into(),
            },
            None,
        );
        fixture.create_user(
            usecases::NewUser {
                email: "admin@foo.tld".into(),
                password: "123456".into(),
            },
            Some(Role::Admin),
        );
        assert_eq!(
            fixture.try_get_user("user@bar.tld").unwrap().role,
            Role::Guest
        );
        assert!(change_user_role(&fixture, "admin@foo.tld", "user@bar.tld", Role::Scout).is_ok());
        assert_eq!(
            fixture.try_get_user("user@bar.tld").unwrap().role,
            Role::Scout
        );
    }
}
