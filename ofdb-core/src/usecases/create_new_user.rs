use passwords::PasswordGenerator;

use super::prelude::*;
use crate::util::validate;

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: EmailAddress,
    pub password: String,
}

pub fn create_new_user<R: UserRepo>(repo: &R, u: NewUser) -> Result<()> {
    // TODO: parse this outside of this fn
    let password = u.password.parse::<Password>()?;
    if !validate::is_valid_email(u.email.as_str()) {
        return Err(Error::EmailAddress);
    }
    if repo.try_get_user_by_email(&u.email)?.is_some() {
        return Err(Error::UserExists);
    }
    let new_user = User {
        email: u.email,
        email_confirmed: false,
        password,
        role: Role::Guest,
    };
    log::debug!("Creating new user: email = {}", new_user.email);
    repo.create_user(&new_user)?;
    Ok(())
}

const PW_GEN: PasswordGenerator = PasswordGenerator {
    length: 8,
    numbers: true,
    lowercase_letters: true,
    uppercase_letters: true,
    symbols: true,
    strict: false,
    exclude_similar_characters: true,
    spaces: false,
};

pub fn create_user_from_email<R>(repo: &R, email: EmailAddress) -> Result<User>
where
    R: UserRepo,
{
    if let Some(user) = repo.try_get_user_by_email(&email)? {
        return Ok(user);
    }
    // Create a new user with a generated password
    let password = PW_GEN
        .generate_one()
        .expect("Could not generate a new password");
    let new_user = NewUser {
        email: email.clone(),
        password,
    };
    create_new_user(repo, new_user)?;
    Ok(repo.get_user_by_email(&email)?)
}

#[cfg(test)]
mod tests {

    use super::{
        super::{tests::MockDb, *},
        *,
    };

    #[test]
    fn create_two_users() {
        let db = MockDb::default();
        let u = NewUser {
            email: "foo@bar.de".parse().unwrap(),
            password: "secret1".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(db
            .get_user_by_email(&EmailAddress::new_unchecked("foo@bar.de".to_string()))
            .is_ok());
        assert!(db
            .try_get_user_by_email(&EmailAddress::new_unchecked("baz@bar.de".to_string()))
            .unwrap()
            .is_none());

        let u = NewUser {
            email: "baz@bar.de".parse().unwrap(),
            password: "secret2".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(db
            .get_user_by_email(&EmailAddress::new_unchecked("foo@bar.de".to_string()))
            .is_ok());
        assert!(db
            .get_user_by_email(&EmailAddress::new_unchecked("baz@bar.de".to_string()))
            .is_ok());
    }

    #[test]
    fn create_user_with_invalid_password() {
        let db = MockDb::default();
        let u = NewUser {
            email: "foo@baz.io".parse().unwrap(),
            password: "hello".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: "foo@baz.io".parse().unwrap(),
            password: "valid pass".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_email() {
        let db = MockDb::default();
        let u = NewUser {
            email: EmailAddress::new_unchecked("".into()),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: EmailAddress::new_unchecked("fooo@".into()),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: EmailAddress::new_unchecked("fooo@bar.io".into()),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
    }

    #[test]
    fn create_user_with_existing_email() {
        let db = MockDb::default();
        db.users.borrow_mut().push(User {
            email: EmailAddress::new_unchecked("baz@foo.bar".to_string()),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        });
        let u = NewUser {
            email: EmailAddress::new_unchecked("baz@foo.bar".to_string()),
            password: "secret".into(),
        };
        match create_new_user(&db, u).err().unwrap() {
            Error::UserExists => {
                // ok
            }
            _ => panic!("invalid error"),
        }
    }

    #[test]
    fn email_unconfirmed_on_default() {
        let db = MockDb::default();
        let u = NewUser {
            email: EmailAddress::new_unchecked("foo@bar.io".to_string()),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(!db.users.borrow()[0].email_confirmed);
    }

    #[test]
    fn encrypt_user_password() {
        let db = MockDb::default();
        let u = NewUser {
            email: EmailAddress::new_unchecked("foo@bar.io".to_string()),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(db.users.borrow()[0].password.as_ref() != "secret");
        assert!(db.users.borrow()[0].password.verify("secret"));
    }

    #[test]
    fn test_create_user_from_email() {
        let db = MockDb::default();
        assert_eq!(
            "mail@tld.com",
            create_user_from_email(&db, EmailAddress::new_unchecked("mail@tld.com".to_string()))
                .unwrap()
                .email
                .as_str(),
        );
    }
}
