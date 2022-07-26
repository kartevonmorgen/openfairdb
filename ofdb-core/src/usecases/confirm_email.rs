use super::prelude::*;

pub fn confirm_email_address<R>(repo: &R, token: &str) -> Result<()>
where
    R: UserRepo,
{
    let email_nonce = EmailNonce::decode_from_str(token).map_err(|_| Error::TokenInvalid)?;
    let mut user = repo.get_user_by_email(&email_nonce.email)?;
    if !user.email_confirmed {
        user.email_confirmed = true;
        debug_assert_eq!(Role::Guest, user.role);
        if user.role == Role::Guest {
            user.role = Role::User;
        }
        repo.update_user(&user)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{super::tests::MockDb, *};

    #[test]
    fn confirm_email_of_existing_user() {
        let db = MockDb::default();
        let email = "a@foo.bar";
        db.users.borrow_mut().push(User {
            email: email.into(),
            email_confirmed: false,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        });
        let email_nonce = EmailNonce {
            email: email.into(),
            nonce: Nonce::new(),
        };
        assert!(confirm_email_address(&db, &email_nonce.encode_to_string()).is_ok());
        assert!(db.users.borrow()[0].email_confirmed);
        assert_eq!(db.users.borrow()[0].role, Role::User);
    }
}
