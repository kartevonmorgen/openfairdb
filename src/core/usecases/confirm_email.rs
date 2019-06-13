use crate::core::prelude::*;

pub fn confirm_email_address(db: &dyn Db, u_id: &str) -> Result<()> {
    //TODO: use username instead of user ID
    let mut u = db
        .all_users()?
        .into_iter()
        .find(|u| u.id == u_id)
        .ok_or_else(|| Error::Repo(RepoError::NotFound))?;
    u.email_confirmed = true;
    u.role = Role::User;
    db.update_user(&u)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::MockDb;
    use super::*;

    #[test]
    fn confirm_email_of_existing_user() {
        let mut db = MockDb::default();
        db.users.borrow_mut().push(User {
            id: "1".into(),
            username: "a".into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "a@foo.bar".into(),
            email_confirmed: false,
            role: Role::Guest,
        });
        assert!(confirm_email_address(&mut db, "1").is_ok());
        assert_eq!(db.users.borrow()[0].email_confirmed, true);
        assert_eq!(db.users.borrow()[0].role, Role::User);
    }
}
