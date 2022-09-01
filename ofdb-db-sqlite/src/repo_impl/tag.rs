use super::*;

impl TagRepo for Connection<'_> {
    fn create_tag_if_it_does_not_exist(&self, t: &Tag) -> Result<()> {
        let res = diesel::insert_into(schema::tags::table)
            .values(&models::Tag::from(t.clone()))
            .execute(self.deref());
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
    fn all_tags(&self) -> Result<Vec<Tag>> {
        use schema::tags::dsl::*;
        Ok(tags
            .load::<models::Tag>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Tag::from)
            .collect())
    }
    fn count_tags(&self) -> Result<usize> {
        use schema::tags::dsl::*;
        Ok(tags
            .select(diesel::dsl::count(id))
            .first::<i64>(self.deref())
            .map_err(from_diesel_err)? as usize)
    }
}
