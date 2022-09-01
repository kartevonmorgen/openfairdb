use super::*;

impl<'a> TagRepo for DbReadOnly<'a> {
    fn create_tag_if_it_does_not_exist(&self, _tag: &Tag) -> Result<()> {
        unreachable!();
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        all_tags(&mut self.conn.borrow_mut())
    }
    fn count_tags(&self) -> Result<usize> {
        count_tags(&mut self.conn.borrow_mut())
    }
}

impl<'a> TagRepo for DbReadWrite<'a> {
    fn create_tag_if_it_does_not_exist(&self, tag: &Tag) -> Result<()> {
        create_tag_if_it_does_not_exist(&mut self.conn.borrow_mut(), tag)
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        all_tags(&mut self.conn.borrow_mut())
    }
    fn count_tags(&self) -> Result<usize> {
        count_tags(&mut self.conn.borrow_mut())
    }
}

impl<'a> TagRepo for DbConnection<'a> {
    fn create_tag_if_it_does_not_exist(&self, tag: &Tag) -> Result<()> {
        create_tag_if_it_does_not_exist(&mut self.conn.borrow_mut(), tag)
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        all_tags(&mut self.conn.borrow_mut())
    }
    fn count_tags(&self) -> Result<usize> {
        count_tags(&mut self.conn.borrow_mut())
    }
}

fn create_tag_if_it_does_not_exist(conn: &mut SqliteConnection, t: &Tag) -> Result<()> {
    let res = diesel::insert_into(schema::tags::table)
        .values(&models::Tag::from(t.clone()))
        .execute(conn);
    if let Err(err) = res {
        match err {
            DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                // that's ok :)
            }
            _ => {
                return Err(from_diesel_err(err));
            }
        }
    }
    Ok(())
}

fn all_tags(conn: &mut SqliteConnection) -> Result<Vec<Tag>> {
    use schema::tags::dsl::*;
    Ok(tags
        .load::<models::Tag>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(Tag::from)
        .collect())
}

fn count_tags(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::tags::dsl::*;
    Ok(tags
        .select(diesel::dsl::count(id))
        .first::<i64>(conn)
        .map_err(from_diesel_err)? as usize)
}
