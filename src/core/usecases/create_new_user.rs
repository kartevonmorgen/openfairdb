use super::super::util::validate;
use crate::core::prelude::*;
use passwords::PasswordGenerator;
use pwhash::bcrypt;
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

pub fn create_user_from_email<D: Db>(db: &mut D, email: &str) -> Result<String> {
    let users: Vec<_> = db.all_users()?;
    let username = match users.iter().find(|u| u.email == email) {
        Some(u) => u.username.clone(),
        None => {
            let generated_username = slugify(&email).replace("-", "");
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
            role: Role::Guest,
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
