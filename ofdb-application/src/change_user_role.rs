use super::*;

pub fn change_user_role(
    connections: &sqlite::Connections,
    account_email: &EmailAddress,
    user_email: &EmailAddress,
    role: Role,
) -> Result<()> {
    Ok(connections.exclusive()?.transaction(|conn| {
        usecases::change_user_role(conn, account_email, user_email, role).map_err(|err| {
            log::warn!("Failed to change role for email {}: {}", user_email, err);
            err
        })
    })?)
}

#[cfg(test)]
mod tests {
    use super::super::tests::prelude::*;
    use std::str::FromStr;

    fn change_user_role(
        fixture: &BackendFixture,
        account_email: &EmailAddress,
        user_email: &EmailAddress,
        role: Role,
    ) -> super::Result<()> {
        super::change_user_role(&fixture.db_connections, account_email, user_email, role)
    }

    #[test]
    fn should_change_the_role_to_scout_if_its_done_by_an_admin() {
        let fixture = BackendFixture::new();
        let user_email = EmailAddress::from_str("user@bar.tld").unwrap();
        let admin_email = EmailAddress::from_str("admin@bar.tld").unwrap();

        fixture.create_user(
            usecases::NewUser {
                email: user_email.clone(),
                password: "123456".into(),
            },
            None,
        );
        fixture.create_user(
            usecases::NewUser {
                email: admin_email.clone(),
                password: "123456".into(),
            },
            Some(Role::Admin),
        );
        assert_eq!(fixture.try_get_user(&user_email).unwrap().role, Role::Guest);
        assert!(change_user_role(&fixture, &admin_email, &user_email, Role::Scout).is_ok());
        assert_eq!(fixture.try_get_user(&user_email).unwrap().role, Role::Scout);
    }
}
