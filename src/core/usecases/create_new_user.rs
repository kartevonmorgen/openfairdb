use super::super::util::validate;
use crate::core::prelude::*;
use pwhash::bcrypt;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

pub fn create_new_user<D: UserGateway>(db: &mut D, u: NewUser) -> Result<()> {
    validate::username(&u.username)?;
    validate::password(&u.password)?;
    validate::email(&u.email)?;
    if db.get_user(&u.username).is_ok() {
        return Err(Error::Parameter(ParameterError::UserExists));
    }
    let new_user = User {
        id: Uuid::new_v4().to_simple_ref().to_string(),
        username: u.username,
        password: bcrypt::hash(&u.password)?,
        email: u.email,
        email_confirmed: false,
        access: AccessLevel::User,
    };
    debug!("Creating new user: {:?}", new_user);
    db.create_user(&new_user)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::super::*;
    use super::*;

    #[test]
    fn create_two_users() {
        let mut db = MockDb::new();
        let u = NewUser {
            username: "foo".into(),
            password: "bar".into(),
            email: "foo@bar.de".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        let u = NewUser {
            username: "baz".into(),
            password: "bar".into(),
            email: "baz@bar.de".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());

        let (foo_username, _) = get_user(&mut db, "foo", "foo").unwrap();
        let (baz_username, _) = get_user(&mut db, "baz", "baz").unwrap();
        assert_eq!(foo_username, "foo");
        assert_eq!(baz_username, "baz");
    }

    #[test]
    fn create_user_with_invalid_name() {
        let mut db = MockDb::new();
        let u = NewUser {
            username: "".into(),
            password: "bar".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "also&invalid".into(),
            password: "bar".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "thisisvalid".into(),
            password: "very_secret".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_password() {
        let mut db = MockDb::new();
        let u = NewUser {
            username: "user".into(),
            password: "".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "not valid".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "validpass".into(),
            email: "foo@baz.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_invalid_email() {
        let mut db = MockDb::new();
        let u = NewUser {
            username: "user".into(),
            password: "pass".into(),
            email: "".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "pass".into(),
            email: "fooo@".into(),
        };
        assert!(create_new_user(&mut db, u).is_err());
        let u = NewUser {
            username: "user".into(),
            password: "pass".into(),
            email: "fooo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
    }

    #[test]
    fn create_user_with_existing_username() {
        let mut db = MockDb::new();
        db.users = vec![User {
            id: "123".into(),
            username: "foo".into(),
            password: "bar".into(),
            email: "baz@foo.bar".into(),
            email_confirmed: true,
            access: AccessLevel::User,
        }];
        let u = NewUser {
            username: "foo".into(),
            password: "pass".into(),
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
        let mut db = MockDb::new();
        let u = NewUser {
            username: "user".into(),
            password: "pass".into(),
            email: "foo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        assert_eq!(db.users[0].email_confirmed, false);
    }

    #[test]
    fn encrypt_user_password() {
        let mut db = MockDb::new();
        let u = NewUser {
            username: "user".into(),
            password: "pass".into(),
            email: "foo@bar.io".into(),
        };
        assert!(create_new_user(&mut db, u).is_ok());
        assert!(db.users[0].password != "pass");
        assert!(bcrypt::verify("pass", &db.users[0].password));
    }

}
