use crate::core::prelude::*;

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone)]
pub struct RateEntry {
    pub entry   : String,
    pub title   : String,
    pub value   : RatingValue,
    pub context : RatingContext,
    pub comment : String,
    pub source  : Option<String>,
    pub user    : Option<String>,
}

#[derive(Debug, Clone)]
pub struct Storable(PlaceRev, Rating, Comment);

impl Storable {
    pub fn rating_uid(&self) -> &str {
        &self.1.uid.as_ref()
    }
    pub fn comment_uid(&self) -> &str {
        &self.2.uid.as_ref()
    }
}

pub fn prepare_new_rating<D: Db>(db: &D, r: RateEntry) -> Result<Storable> {
    if r.comment.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyComment));
    }
    if !r.value.is_valid() {
        return Err(Error::Parameter(ParameterError::RatingValue));
    }
    let now = Timestamp::now();
    let rating_uid = Uid::new_uuid();
    let comment_uid = Uid::new_uuid();
    let (place_rev, _) = db.get_place(&r.entry)?;
    debug_assert_eq!(place_rev.uid, r.entry.as_str().into());
    let rating = Rating {
        uid: rating_uid.clone(),
        place_uid: r.entry.into(),
        created_at: now,
        archived_at: None,
        title: r.title,
        value: r.value,
        context: r.context,
        source: r.source,
    };
    let comment = Comment {
        uid: comment_uid,
        rating_uid,
        created_at: now,
        archived_at: None,
        text: r.comment,
    };
    Ok(Storable(place_rev, rating, comment))
}

pub fn store_new_rating<D: Db>(db: &D, s: Storable) -> Result<(PlaceRev, Vec<Rating>)> {
    let Storable(place_rev, rating, comment) = s;
    debug_assert_eq!(place_rev.uid, rating.place_uid);
    debug_assert_eq!(rating.uid, comment.rating_uid);
    db.create_rating(rating)?;
    db.create_comment(comment)?;
    let ratings = db.load_ratings_of_entry(place_rev.uid.as_ref())?;
    Ok((place_rev, ratings))
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
            RateEntry {
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
        let e = PlaceRev::build().id("foo").finish();
        db.entries = vec![(e, Status::created())].into();
        assert!(prepare_new_rating(
            &db,
            RateEntry {
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
        let e = PlaceRev::build().id("foo").finish();
        db.entries = vec![(e, Status::created())].into();
        assert!(prepare_new_rating(
            &db,
            RateEntry {
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
            RateEntry {
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
        let e = PlaceRev::build().id("foo").finish();
        db.entries = vec![(e, Status::created())].into();
        let c = prepare_new_rating(
            &db,
            RateEntry {
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
        assert_eq!(db.ratings.borrow()[0].place_uid, "foo".into());
        assert_eq!(
            db.comments.borrow()[0].rating_uid,
            db.ratings.borrow()[0].uid
        );
    }
}
