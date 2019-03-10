use super::super::util::validate;
use crate::core::prelude::*;
use passwords::PasswordGenerator;
use slug::slugify;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub fn create_new_user<D: UserGateway>(db: &mut D, u: NewUser) -> Result<()> {
    validate::username(&u.username)?;
    let password = u.password.parse::<Password>()?;
    validate::email(&u.email)?;
    if db.get_user(&u.username).is_ok() {
        return Err(Error::Parameter(ParameterError::UserExists));
    }
    let new_user = User {
        id: Uuid::new_v4().to_simple_ref().to_string(),
        username: u.username,
        password,
        email: u.email,
        email_confirmed: false,
        role: Role::Guest,
    };
    debug!(
        "Creating new user: username = {}, email = {}, ",
        new_user.username, new_user.email
    );
    db.create_user(new_user)?;
    Ok(())
}

const PW_GEN: PasswordGenerator = PasswordGenerator {
    length: 8,
    numbers: true,
    lowercase_letters: true,
    uppercase_letters: true,
    symbols: true,
    strict: false,
};

pub const MAX_USERNAME_LEN: usize = 40;

pub fn generate_username_from_email(email: &str) -> String {
    let mut generated_username = slugify(&email).replace("-", "");
    generated_username.truncate(MAX_USERNAME_LEN);
    generated_username
}

pub fn create_user_from_email<D: Db>(db: &mut D, email: &str) -> Result<String> {
    let users: Vec<_> = db.all_users()?;
    let username = match users.iter().find(|u| u.email == email) {
        Some(u) => u.username.clone(),
        None => {
            let generated_username = generate_username_from_email(&email);
            let username = generated_username.clone();
            let password = PW_GEN.generate_one().map_err(|e| e.to_string())?;
            let u = NewUser {
                username,
                password,
                email: email.into(),
            };
            create_new_user(db, u)?;
            generated_username
        }
    };
    Ok(username)
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::super::*;
    use super::*;

    #[test]
    fn create_two_users() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "foo".into(),
            password: "secret1".into(),
            email: "foo@bar.de".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        let u = NewUser {
            username: "baz".into(),
            password: "secret2".into(),
            email: "baz@bar.de".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());

        let (foo_username, _) = get_user(&db, "foo", "foo").unwrap();
        let (baz_username, _) = get_user(&db, "baz", "baz").unwrap();
        assert_eq!(foo_username, "foo");
        assert_eq!(baz_username, "baz");
    }

    #[test]
    fn create_user_with_invalid_name() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "".into(),
            password: "secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "invalid&username".into(),
            password: "secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "invalid_username".into(),
            password: "secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "invalid username".into(),
            password: "secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "0validusername12".into(),
            password: "very secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_password() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "user".into(),
            password: "hello".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "valid pass".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_email() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "user".into(),
            password: "secret".into(),
            email: "".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "secret".into(),
            email: "fooo@".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "secret".into(),
            email: "fooo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_existing_username() {
        let mut db = MockDb::default();
        db.users.borrow_mut().push(User {
            id: "123".into(),
            username: "foo".into(),
            password: "secret".parse::<Password>().unwrap(),
            email: "baz@foo.bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        });
        let u = NewUser {
            username: "foo".into(),
            password: "secret".into(),
            email: "user@server.tld".into(),
        };
        match create_new_user(&mut db, u).err().unwrap() {
            Error::Parameter(err) => {
                match err {
                    ParameterError::UserExists => {
                        // ok
                    }
                    _ => panic!("invalid error"),
                }
            }
            _ => panic!("invalid error"),
        }
    }

    #[test]
    fn email_unconfirmed_on_default() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "user".into(),
            password: "secret".into(),
            email: "foo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        assert_eq!(db.users.borrow()[0].email_confirmed, false);
    }

    #[test]
    fn encrypt_user_password() {
        let mut db = MockDb::default();
        let u = NewUser {
            username: "user".into(),
            password: "secret".into(),
            email: "foo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        assert!(db.users.borrow()[0].password.as_ref() != "secret");
        assert!(db.users.borrow()[0].password.verify("secret"));
    }

    #[test]
    fn test_create_user_from_email() {
        let mut db = MockDb::default();
        assert_eq!(
            create_user_from_email(&mut db, "mail@tld.com").unwrap(),
            "mailtldcom"
        );
        assert_eq!(
            create_user_from_email(
                &mut db,
                "a-very-very-long-email@with-a-very-very-long-domain.com"
            )
            .unwrap(),
            "averyverylongemailwithaveryverylongdomai"
        );
    }
}
