use passwords::PasswordGenerator;

use super::super::util::validate;
use crate::core::prelude::*;

#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub password: String,
}

pub fn create_new_user<D: UserGateway>(db: &D, u: NewUser) -> Result<()> {
    let password = u.password.parse::<Password>()?;
    if !validate::is_valid_email(&u.email) {
        return Err(ParameterError::Email.into());
    }
    if db.try_get_user_by_email(&u.email)?.is_some() {
        return Err(ParameterError::UserExists.into());
    }
    let new_user = User {
        email: u.email,
        email_confirmed: false,
        password,
        role: Role::Guest,
    };
    debug!("Creating new user: email = {}", new_user.email);
    db.create_user(&new_user)?;
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

pub fn create_user_from_email<D: Db>(db: &D, email: &str) -> Result<User> {
    if let Some(user) = db.try_get_user_by_email(email)? {
        return Ok(user);
    }
    // Create a new user with a generated password
    let password = PW_GEN.generate_one().map_err(ToString::to_string)?;
    let u = NewUser {
        email: email.into(),
        password,
    };
    create_new_user(db, u)?;
    Ok(db.get_user_by_email(email)?)
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
            email: "foo@bar.de".into(),
            password: "secret1".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(db.get_user_by_email("foo@bar.de").is_ok());
        assert!(db.try_get_user_by_email("baz@bar.de").unwrap().is_none());

        let u = NewUser {
            email: "baz@bar.de".into(),
            password: "secret2".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(db.get_user_by_email("foo@bar.de").is_ok());
        assert!(db.get_user_by_email("baz@bar.de").is_ok());
    }

    #[test]
    fn create_user_with_invalid_password() {
        let db = MockDb::default();
        let u = NewUser {
            email: "foo@baz.io".into(),
            password: "hello".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: "foo@baz.io".into(),
            password: "valid pass".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_email() {
        let db = MockDb::default();
        let u = NewUser {
            email: "".into(),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: "fooo@".into(),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_err());
        let u = NewUser {
            email: "fooo@bar.io".into(),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
    }

    #[test]
    fn create_user_with_existing_email() {
        let db = MockDb::default();
        db.users.borrow_mut().push(User {
            email: "baz@foo.bar".into(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Guest,
        });
        let u = NewUser {
            email: "baz@foo.bar".into(),
            password: "secret".into(),
        };
        match create_new_user(&db, u).err().unwrap() {
            Error::Parameter(ParameterError::UserExists) => {
                // ok
            }
            _ => panic!("invalid error"),
        }
    }

    #[test]
    fn email_unconfirmed_on_default() {
        let db = MockDb::default();
        let u = NewUser {
            email: "foo@bar.io".into(),
            password: "secret".into(),
        };
        assert!(create_new_user(&db, u).is_ok());
        assert!(!db.users.borrow()[0].email_confirmed);
    }

    #[test]
    fn encrypt_user_password() {
        let db = MockDb::default();
        let u = NewUser {
            email: "foo@bar.io".into(),
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
            create_user_from_email(&db, "mail@tld.com").unwrap().email,
        );
    }
}
