use super::*;

impl RatingRepository for Connection<'_> {
    fn create_rating(&self, rating: Rating) -> Result<()> {
        let Rating {
            id,
            place_id,
            created_at,
            archived_at,
            title,
            value,
            context,
            source,
        } = rating;
        let parent_rowid = resolve_place_rowid(self, &place_id)?;
        let new_place_rating = models::NewPlaceRating {
            id: id.into(),
            parent_rowid,
            created_at: created_at.as_millis(),
            created_by: None,
            archived_at: archived_at.map(Timestamp::as_millis),
            archived_by: None,
            title,
            value: i8::from(value).into(),
            context: util::rating_context_to_string(context),
            source,
        };
        let _count = diesel::insert_into(schema::place_rating::table)
            .values(&new_place_rating)
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>> {
        use schema::{place::dsl, place_rating::dsl as rating_dsl};
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::rowid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::id,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
                dsl::id,
            ))
            .filter(rating_dsl::id.eq_any(ids))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_rating(&self, id: &str) -> Result<Rating> {
        let ratings = self.load_ratings(&[id])?;
        debug_assert!(ratings.len() <= 1);
        ratings.into_iter().next().ok_or(repo::Error::NotFound)
    }

    fn load_ratings_of_place(&self, place_id: &str) -> Result<Vec<Rating>> {
        use schema::{place::dsl, place_rating::dsl as rating_dsl};
        Ok(schema::place_rating::table
            .inner_join(schema::place::table)
            .select((
                rating_dsl::rowid,
                rating_dsl::created_at,
                rating_dsl::created_by,
                rating_dsl::archived_at,
                rating_dsl::archived_by,
                rating_dsl::id,
                rating_dsl::title,
                rating_dsl::value,
                rating_dsl::context,
                rating_dsl::source,
                dsl::id,
            ))
            .filter(dsl::id.eq(place_id))
            .filter(rating_dsl::archived_at.is_null())
            .load::<models::PlaceRating>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_place_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>> {
        use schema::{place::dsl, place_rating::dsl as rating_dsl};
        schema::place_rating::table
            .inner_join(schema::place::table)
            .select(dsl::id)
            .filter(rating_dsl::id.eq_any(ids))
            .load::<String>(self.deref())
            .map_err(from_diesel_err)
    }

    fn archive_ratings(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place_rating::dsl;
        let archived_at = Some(activity.at.as_millis());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        let count = diesel::update(
            schema::place_rating::table
                .filter(dsl::id.eq_any(ids))
                .filter(dsl::archived_at.is_null()),
        )
        .set((
            dsl::archived_at.eq(archived_at),
            dsl::archived_by.eq(archived_by),
        ))
        .execute(self.deref())
        .map_err(from_diesel_err)?;
        debug_assert!(count <= ids.len());
        Ok(count)
    }

    fn archive_ratings_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::{place::dsl, place_rating::dsl as rating_dsl};
        let archived_at = Some(activity.at.as_millis());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        diesel::update(
            schema::place_rating::table
                .filter(
                    rating_dsl::parent_rowid.eq_any(
                        schema::place::table
                            .select(dsl::rowid)
                            .filter(dsl::id.eq_any(place_ids)),
                    ),
                )
                .filter(rating_dsl::archived_at.is_null()),
        )
        .set((
            rating_dsl::archived_at.eq(archived_at),
            rating_dsl::archived_by.eq(archived_by),
        ))
        .execute(self.deref())
        .map_err(from_diesel_err)
    }
}
