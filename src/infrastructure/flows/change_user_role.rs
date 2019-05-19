use super::*;
use diesel::connection::Connection;

pub fn change_user_role(
    connections: &sqlite::Connections,
    account_email: &str,
    user_email: &str,
    role: Role,
) -> Result<()> {
    let mut repo_err = None;
    let connection = connections.exclusive()?;
    Ok(connection
        .transaction::<_, diesel::result::Error, _>(|| {
            usecases::change_user_role(&*connection, account_email, user_email, role).map_err(
                |err| {
                    warn!("Failed to chage role for email {}: {}", user_email, err);
                    repo_err = Some(err);
                    diesel::result::Error::RollbackTransaction
                },
            )
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

    fn change_user_role(
        fixture: &EnvFixture,
        account_email: &str,
        user_email: &str,
        role: Role,
    ) -> super::Result<()> {
        super::change_user_role(&fixture.db_connections, account_email, user_email, role)
    }

    #[test]
    fn should_change_the_role_to_scout_if_its_done_by_an_admin() {
        let fixture = EnvFixture::new();
        fixture.create_user(
            usecases::NewUser {
                email: "user@bar.tld".into(),
                password: "123456".into(),
                username: "user".into(),
            },
            None,
        );
        fixture.create_user(
            usecases::NewUser {
                email: "admin@foo.tld".into(),
                password: "123456".into(),
                username: "admin".into(),
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
