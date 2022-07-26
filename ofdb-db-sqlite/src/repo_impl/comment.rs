use super::*;

impl CommentRepository for Connection<'_> {
    fn create_comment(&self, comment: Comment) -> Result<()> {
        let Comment {
            id,
            rating_id,
            created_at,
            archived_at,
            text,
            ..
        } = comment;
        let parent_rowid = resolve_rating_rowid(self, rating_id.as_ref())?;
        let new_place_rating_comment = models::NewPlaceRatingComment {
            id: id.into(),
            parent_rowid,
            created_at: created_at.as_millis(),
            created_by: None,
            archived_at: archived_at.map(Timestamp::as_millis),
            archived_by: None,
            text,
        };
        let _count = diesel::insert_into(schema::place_rating_comment::table)
            .values(&new_place_rating_comment)
            .execute(self.deref())
            .map_err(from_diesel_err)?;
        debug_assert_eq!(1, _count);
        Ok(())
    }

    fn load_comments(&self, ids: &[&str]) -> Result<Vec<Comment>> {
        use schema::{place_rating::dsl as rating_dsl, place_rating_comment::dsl as comment_dsl};
        // TODO: Split loading into chunks of fixed size
        log::info!("Loading multiple ({}) comments at once", ids.len());
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::rowid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::id,
                comment_dsl::text,
                rating_dsl::id,
            ))
            .filter(comment_dsl::id.eq_any(ids))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn load_comment(&self, id: &str) -> Result<Comment> {
        let comments = self.load_comments(&[id])?;
        debug_assert!(comments.len() <= 1);
        comments.into_iter().next().ok_or(repo::Error::NotFound)
    }

    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>> {
        use schema::{place_rating::dsl as rating_dsl, place_rating_comment::dsl as comment_dsl};
        Ok(schema::place_rating_comment::table
            .inner_join(schema::place_rating::table)
            .select((
                comment_dsl::rowid,
                comment_dsl::created_at,
                comment_dsl::created_by,
                comment_dsl::archived_at,
                comment_dsl::archived_by,
                comment_dsl::id,
                comment_dsl::text,
                rating_dsl::id,
            ))
            .filter(rating_dsl::id.eq(rating_id))
            .filter(comment_dsl::archived_at.is_null())
            .load::<models::PlaceRatingComment>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    fn archive_comments(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::place_rating_comment::dsl;
        let archived_at = Some(activity.at.as_millis());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        let count = diesel::update(
            schema::place_rating_comment::table
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

    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        activity: &Activity,
    ) -> Result<usize> {
        use schema::{place_rating::dsl as rating_dsl, place_rating_comment::dsl as comment_dsl};
        let archived_at = Some(activity.at.as_millis());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::parent_rowid.eq_any(
                        schema::place_rating::table
                            .select(rating_dsl::rowid)
                            .filter(rating_dsl::id.eq_any(rating_ids)),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(archived_at),
            comment_dsl::archived_by.eq(archived_by),
        ))
        .execute(self.deref())
        .map_err(from_diesel_err)
    }

    fn archive_comments_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        use schema::{
            place::dsl, place_rating::dsl as rating_dsl, place_rating_comment::dsl as comment_dsl,
        };
        let archived_at = Some(activity.at.as_millis());
        let archived_by = if let Some(ref email) = activity.by {
            Some(resolve_user_created_by_email(self, email.as_ref())?)
        } else {
            None
        };
        Ok(diesel::update(
            schema::place_rating_comment::table
                .filter(
                    comment_dsl::parent_rowid.eq_any(
                        schema::place_rating::table
                            .select(rating_dsl::rowid)
                            .filter(
                                rating_dsl::parent_rowid.eq_any(
                                    schema::place::table
                                        .select(dsl::rowid)
                                        .filter(dsl::id.eq_any(place_ids)),
                                ),
                            ),
                    ),
                )
                .filter(comment_dsl::archived_at.is_null()),
        )
        .set((
            comment_dsl::archived_at.eq(archived_at),
            comment_dsl::archived_by.eq(archived_by),
        ))
        .execute(self.deref())
        .optional()
        .map_err(from_diesel_err)?
        .unwrap_or_default())
    }
}
