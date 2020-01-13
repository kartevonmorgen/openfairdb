use crate::core::prelude::*;

#[derive(Debug, Clone)]
pub struct Storable(Place, ReviewStatus, Rating, Comment);

impl Storable {
    pub fn rating_id(&self) -> &str {
        &self.2.id.as_ref()
    }
    pub fn comment_id(&self) -> &str {
        &self.3.id.as_ref()
    }
}

pub fn prepare_new_rating<D: Db>(db: &D, r: NewPlaceRating) -> Result<Storable> {
    if r.comment.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyComment));
    }
    if !r.value.is_valid() {
        return Err(Error::Parameter(ParameterError::RatingValue));
    }
    let now = Timestamp::now();
    let rating_id = Id::new();
    let comment_id = Id::new();
    let (place, status) = db.get_place(&r.entry)?;
    debug_assert_eq!(place.id, r.entry.as_str().into());
    let rating = Rating {
        id: rating_id.clone(),
        place_id: r.entry.into(),
        created_at: now,
        archived_at: None,
        title: r.title,
        value: r.value,
        context: r.context,
        source: r.source,
    };
    let comment = Comment {
        id: comment_id,
        rating_id,
        created_at: now,
        archived_at: None,
        text: r.comment,
    };
    Ok(Storable(place, status, rating, comment))
}

pub fn store_new_rating<D: Db>(db: &D, s: Storable) -> Result<(Place, ReviewStatus, Vec<Rating>)> {
    let Storable(place, status, rating, comment) = s;
    debug_assert_eq!(place.id, rating.place_id);
    debug_assert_eq!(rating.id, comment.rating_id);
    db.create_rating(rating)?;
    db.create_comment(comment)?;
    let ratings = db.load_ratings_of_place(place.id.as_ref())?;
    Ok((place, status, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::super::*;
    use super::*;

    #[test]
    fn rate_non_existing_entry() {
        let db = MockDb::default();
        assert!(prepare_new_rating(
            &db,
            NewPlaceRating {
                entry: "does_not_exist".into(),
                title: "title".into(),
                comment: "a comment".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_with_empty_comment() {
        let mut db = MockDb::default();
        let place = Place::build().id("foo").finish();
        db.entries = vec![(place, ReviewStatus::Created)].into();
        assert!(prepare_new_rating(
            &db,
            NewPlaceRating {
                entry: "foo".into(),
                comment: "".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_with_invalid_value_comment() {
        let mut db = MockDb::default();
        let p = Place::build().id("foo").finish();
        db.entries = vec![(p, ReviewStatus::Created)].into();
        assert!(prepare_new_rating(
            &db,
            NewPlaceRating {
                entry: "foo".into(),
                comment: "comment".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(3),
                source: Some("source".into()),
            },
        )
        .is_err());
        assert!(prepare_new_rating(
            &db,
            NewPlaceRating {
                entry: "foo".into(),
                title: "title".into(),
                comment: "comment".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(-2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_without_login() {
        let mut db = MockDb::default();
        let p = Place::build().id("foo").finish();
        db.entries = vec![(p, ReviewStatus::Created)].into();
        let c = prepare_new_rating(
            &db,
            NewPlaceRating {
                entry: "foo".into(),
                comment: "comment".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .unwrap();
        assert!(store_new_rating(&db, c).is_ok());

        assert_eq!(db.ratings.borrow().len(), 1);
        assert_eq!(db.comments.borrow().len(), 1);
        assert_eq!(db.ratings.borrow()[0].place_id, "foo".into());
        assert_eq!(db.comments.borrow()[0].rating_id, db.ratings.borrow()[0].id);
    }
}
