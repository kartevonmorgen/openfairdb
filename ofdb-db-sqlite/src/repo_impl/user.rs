use super::*;

impl<'a> UserRepo for DbReadOnly<'a> {
    fn create_user(&self, _user: &User) -> Result<()> {
        unreachable!();
    }
    fn update_user(&self, _user: &User) -> Result<()> {
        unreachable!();
    }
    fn delete_user_by_email(&self, _email: &str) -> Result<()> {
        unreachable!();
    }

    fn all_users(&self) -> Result<Vec<User>> {
        all_users(&mut self.conn.borrow_mut())
    }
    fn count_users(&self) -> Result<usize> {
        count_users(&mut self.conn.borrow_mut())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        try_get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
}

impl<'a> UserRepo for DbReadWrite<'a> {
    fn create_user(&self, user: &User) -> Result<()> {
        create_user(&mut self.conn.borrow_mut(), user)
    }
    fn update_user(&self, user: &User) -> Result<()> {
        update_user(&mut self.conn.borrow_mut(), user)
    }
    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        delete_user_by_email(&mut self.conn.borrow_mut(), email)
    }

    fn all_users(&self) -> Result<Vec<User>> {
        all_users(&mut self.conn.borrow_mut())
    }
    fn count_users(&self) -> Result<usize> {
        count_users(&mut self.conn.borrow_mut())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        try_get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
}

impl<'a> UserRepo for DbConnection<'a> {
    fn create_user(&self, user: &User) -> Result<()> {
        create_user(&mut self.conn.borrow_mut(), user)
    }
    fn update_user(&self, user: &User) -> Result<()> {
        update_user(&mut self.conn.borrow_mut(), user)
    }
    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        delete_user_by_email(&mut self.conn.borrow_mut(), email)
    }

    fn all_users(&self) -> Result<Vec<User>> {
        all_users(&mut self.conn.borrow_mut())
    }
    fn count_users(&self) -> Result<usize> {
        count_users(&mut self.conn.borrow_mut())
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        try_get_user_by_email(&mut self.conn.borrow_mut(), email)
    }
}

fn create_user(conn: &mut SqliteConnection, u: &User) -> Result<()> {
    let new_user = models::NewUser::from(u);
    diesel::insert_into(schema::users::table)
        .values(&new_user)
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(())
}

fn update_user(conn: &mut SqliteConnection, u: &User) -> Result<()> {
    use schema::users::dsl;
    let new_user = models::NewUser::from(u);
    diesel::update(dsl::users.filter(dsl::email.eq(new_user.email)))
        .set(&new_user)
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(())
}

fn delete_user_by_email(conn: &mut SqliteConnection, email: &str) -> Result<()> {
    use schema::users::dsl;
    diesel::delete(dsl::users.filter(dsl::email.eq(email)))
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(())
}

fn get_user_by_email(conn: &mut SqliteConnection, email: &str) -> Result<User> {
    use schema::users::dsl;
    Ok(dsl::users
        .filter(dsl::email.eq(email))
        .first::<models::UserEntity>(conn)
        .map_err(from_diesel_err)?
        .into())
}

fn try_get_user_by_email(conn: &mut SqliteConnection, email: &str) -> Result<Option<User>> {
    use schema::users::dsl;
    Ok(dsl::users
        .filter(dsl::email.eq(email))
        .first::<models::UserEntity>(conn)
        .optional()
        .map_err(from_diesel_err)?
        .map(Into::into))
}

fn all_users(conn: &mut SqliteConnection) -> Result<Vec<User>> {
    use schema::users::dsl;
    Ok(dsl::users
        .load::<models::UserEntity>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(Into::into)
        .collect())
}

fn count_users(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::users::dsl;
    Ok(dsl::users
        .select(diesel::dsl::count(dsl::id))
        .first::<i64>(conn)
        .map_err(from_diesel_err)? as usize)
}
